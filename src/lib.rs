//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]
//!

use std::task::Poll;

use futures_util::{future::BoxFuture, Future, FutureExt, TryFutureExt};
use tower::Service;

// pub mod reqwest_compat;
pub mod middleware;

pub type HttpRequest = http::Request<reqwest::Body>;
pub type HttpResponse = http::Response<reqwest::Body>;

// TODO Use own error type
pub type Error = reqwest_middleware::Error;
pub type Result<T, E = crate::Error> = std::result::Result<T, E>;

trait ExecuteRequest {
    fn execute_request(
        &self,
        req: HttpRequest,
    ) -> crate::Result<impl Future<Output = crate::Result<HttpResponse>> + Send + 'static>;
}

#[derive(Debug, Clone)]
pub struct ReqwestService<E> {
    client: E,
}

impl ExecuteRequest for reqwest::Client {
    fn execute_request(
        &self,
        req: HttpRequest,
    ) -> crate::Result<impl Future<Output = crate::Result<HttpResponse>> + Send + 'static> {
        let reqw: reqwest::Request = req.try_into()?;
        Ok(self
            .execute(reqw)
            .map_ok(HttpResponse::from)
            .map_err(crate::Error::from))
    }
}

impl<E> Service<HttpRequest> for ReqwestService<E>
where
    E: ExecuteRequest,
{
    type Response = HttpResponse;
    type Error = crate::Error;
    // TODO Rewrite without boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::prelude::v1::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        let fut = self.client.execute_request(req);
        async move { fut?.await }.boxed()
    }
}

impl<E> Service<HttpRequest> for &ReqwestService<E>
where
    E: ExecuteRequest,
{
    type Response = HttpResponse;
    type Error = crate::Error;
    // TODO Rewrite without boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::prelude::v1::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        let fut = self.client.execute_request(req);
        async move { fut?.await }.boxed()
    }
}

#[cfg(test)]
mod tests {
    use http::{header::USER_AGENT, HeaderValue};
    use pretty_assertions::assert_eq;
    use reqwest::Client;
    use tower::{Service, ServiceBuilder};
    use tower_http::set_header::SetResponseHeaderLayer;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::ReqwestService;

    #[tokio::test]
    async fn test_dummy_service() -> anyhow::Result<()> {
        // Start a background HTTP server on a random local port
        let mock_server = MockServer::start().await;
        // Get mock server base uri
        let mock_uri = mock_server.uri();

        // Arrange the behaviour of the MockServer adding a Mock:
        // when it receives a GET request on '/hello' it will respond with a 200.
        Mock::given(method("GET"))
            .and(path("/hello"))
            .respond_with(ResponseTemplate::new(200))
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;
        // Create HTTP requests executor
        let mut client = ReqwestService {
            client: Client::new(),
        };

        // Execute request without layers
        let request = http::request::Builder::new()
            .method(http::Method::GET)
            .uri(format!("{mock_uri}/hello"))
            // TODO Improve body manipulations
            .body(reqwest::Body::default())?;
        let response = client.call(request).await?;

        assert!(response.status().is_success());
        // Execute request via ServiceBuilder
        let value = HeaderValue::from_static("tower-reqwest");
        let mut service = ServiceBuilder::new()
            .layer(SetResponseHeaderLayer::overriding(USER_AGENT, value))
            .service(&client);

        let request = http::request::Builder::new()
            .method(http::Method::GET)
            .uri(format!("{mock_uri}/hello"))
            // TODO Improve body manipulations
            .body(reqwest::Body::default())?;
        let response = service.call(request).await?;

        dbg!(&response);
        assert!(response.status().is_success());
        assert_eq!(
            response.headers().get(USER_AGENT).unwrap(),
            HeaderValue::from_static("tower-reqwest")
        );

        Ok(())
    }
}
