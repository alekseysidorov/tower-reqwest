//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]
//!

use std::task::Poll;

use futures_util::{future::BoxFuture, Future, FutureExt, TryFutureExt};
use tower::Service;

pub mod middleware;

pub type HttpResponse = http::Response<reqwest::Body>;

// TODO Use own error type
pub type Error = reqwest_middleware::Error;
pub type Result<T, E = crate::Error> = std::result::Result<T, E>;

trait ExecuteRequest<ReqBody>
where
    ReqBody: Into<reqwest::Body>,
{
    fn execute_request(
        &self,
        req: http::Request<ReqBody>,
    ) -> crate::Result<impl Future<Output = crate::Result<HttpResponse>> + Send + 'static>;
}

#[derive(Debug, Clone)]
pub struct ReqwestService<E> {
    client: E,
}

impl<E> ReqwestService<E> {
    pub fn new(client: E) -> Self {
        Self { client }
    }
}

impl<ReqBody> ExecuteRequest<ReqBody> for reqwest::Client
where
    ReqBody: Into<reqwest::Body>,
{
    fn execute_request(
        &self,
        req: http::Request<ReqBody>,
    ) -> crate::Result<impl Future<Output = crate::Result<HttpResponse>> + Send + 'static> {
        let reqw: reqwest::Request = req.try_into()?;
        Ok(self
            .execute(reqw)
            .map_ok(HttpResponse::from)
            .map_err(crate::Error::from))
    }
}

impl<S, ReqBody> Service<http::Request<ReqBody>> for ReqwestService<S>
where
    S: ExecuteRequest<ReqBody>,
    ReqBody: Into<reqwest::Body>,
{
    type Response = HttpResponse;
    type Error = crate::Error;
    // TODO We need ATPIT to get rid of boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::prelude::v1::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let fut = self.client.execute_request(req);
        async move { fut?.await }.boxed()
    }
}

impl<S, ReqBody> Service<http::Request<ReqBody>> for &ReqwestService<S>
where
    S: ExecuteRequest<ReqBody>,
    ReqBody: Into<reqwest::Body>,
{
    type Response = HttpResponse;
    type Error = crate::Error;
    // TODO We need ATPIT to get rid of boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::prelude::v1::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let fut = self.client.execute_request(req);
        async move { fut?.await }.boxed()
    }
}

#[cfg(test)]
mod tests {
    use http::{header::USER_AGENT, HeaderName, HeaderValue};
    use http_body_util::BodyExt;
    use pretty_assertions::assert_eq;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use tower::{Service, ServiceBuilder};
    use tower_http::{request_id::MakeRequestUuid, ServiceBuilderExt};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::ReqwestService;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Info {
        student: String,
        answer: u32,
        request_id: Option<String>,
    }

    impl Info {
        async fn from_body(body: reqwest::Body) -> anyhow::Result<Self> {
            let body_bytes = body.collect().await?.to_bytes();
            let info: Info = serde_json::from_slice(&body_bytes)?;
            Ok(info)
        }
    }

    #[tokio::test]
    async fn test_service_with_client() -> anyhow::Result<()> {
        // Start a background HTTP server on a random local port
        let mock_server = MockServer::start().await;
        // Get mock server base uri
        let mock_uri = mock_server.uri();

        // Arrange the behaviour of the MockServer adding a Mock:
        // when it receives a GET request on '/hello' it will respond with a 200.
        Mock::given(method("GET"))
            .and(path("/hello"))
            .respond_with(|req: &wiremock::Request| {
                let request_id = req
                    .headers
                    .get(HeaderName::from_static("x-request-id"))
                    .map(|value| value.to_str().unwrap().to_owned());

                ResponseTemplate::new(200).set_body_json(Info {
                    student: "Vasya Pupkin".to_owned(),
                    answer: 42,
                    request_id,
                })
            })
            // Mounting the mock on the mock server - it's now effective!
            .mount(&mock_server)
            .await;
        // Create HTTP requests executor.
        let mut client = ReqwestService::new(Client::new());

        // Execute request without layers
        let request = http::request::Builder::new()
            .method(http::Method::GET)
            .uri(format!("{mock_uri}/hello"))
            // TODO Improve body manipulations
            .body("")?;
        let response = client.call(request.clone()).await?;

        assert!(response.status().is_success());
        // Try to read body
        let info = Info::from_body(response.into_body()).await?;
        assert!(info.request_id.is_none());

        // Execute request via ServiceBuilder
        let mut service = ServiceBuilder::new()
            .override_response_header(USER_AGENT, HeaderValue::from_static("tower-reqwest"))
            .set_x_request_id(MakeRequestUuid)
            .service(&client);
        let response = service.call(request).await?;

        assert!(response.status().is_success());
        assert_eq!(
            response.headers().get(USER_AGENT).unwrap(),
            HeaderValue::from_static("tower-reqwest")
        );

        // Try to read body again.
        let info = Info::from_body(response.into_body()).await?;
        assert_eq!(info.student, "Vasya Pupkin");
        assert_eq!(info.answer, 42);
        assert!(info.request_id.is_some());

        Ok(())
    }
}
