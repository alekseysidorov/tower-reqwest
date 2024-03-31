//! # Overview
//!
#![doc = include_utils::include_md!("README.md:description")]

use std::task::Poll;

use futures_util::{future::BoxFuture, Future, FutureExt, TryFutureExt};
use tower::{Layer, Service};
pub use crate::error::Error;

#[cfg(feature = "reqwest-middleware")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest-middleware")))]
pub mod middleware;
pub mod error;

/// Response type from `http` crate with the body from the `reqwest` crate.
pub type HttpResponse = http::Response<reqwest::Body>;
/// Alias for a Result with the error type `crate::Error`.
pub type Result<T, E = crate::Error> = std::result::Result<T, E>;

#[derive(Debug, Clone)]
pub struct HttpClientService<S>(S);

impl<S> HttpClientService<S> {
    pub fn new(inner: S) -> Self {
        Self(inner)
    }
}

impl<S, ReqBody> Service<http::Request<ReqBody>> for HttpClientService<S>
where
    S: Service<reqwest::Request, Response = reqwest::Response>,
    S::Future: Send + 'static,
    S::Error: 'static,
    crate::Error: From<S::Error>,
    reqwest::Body: From<ReqBody>,
{
    type Response = HttpResponse;
    type Error = crate::Error;
    // TODO We need ATPIT to get rid of boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let fut = execute_request(&mut self.0, req);
        async move { fut?.await }.boxed()
    }
}

fn execute_request<S, B>(
    service: &mut S,
    req: http::Request<B>,
) -> crate::Result<impl Future<Output = crate::Result<HttpResponse>> + Send + 'static>
where
    S: Service<reqwest::Request>,
    S::Future: Send + 'static,
    S::Error: 'static,
    S::Response: 'static,
    crate::Error: From<S::Error>,
    reqwest::Body: From<B>,
    HttpResponse: From<S::Response>,
{
    let reqw = reqwest::Request::try_from(req)?;
    Ok(service
        .call(reqw)
        .map_ok(HttpResponse::from)
        .map_err(crate::Error::from))
}

#[derive(Debug, Clone, Copy)]
pub struct HttpClientLayer;

impl<S> Layer<S> for HttpClientLayer {
    type Service = HttpClientService<S>;

    fn layer(&self, service: S) -> Self::Service {
        HttpClientService(service)
    }
}

#[cfg(test)]
mod tests {
    use http::{header::USER_AGENT, HeaderName, HeaderValue};
    use http_body_util::BodyExt;
    use pretty_assertions::assert_eq;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use tower::{Service, ServiceBuilder, ServiceExt};
    use tower_http::{request_id::MakeRequestUuid, ServiceBuilderExt};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::HttpClientLayer;

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
    async fn test_http_client_layer() -> anyhow::Result<()> {
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
        // Create HTTP client
        let client = Client::new();

        // Execute request without layers
        let request = http::request::Builder::new()
            .method(http::Method::GET)
            .uri(format!("{mock_uri}/hello"))
            // TODO Make in easy to create requests without body.
            .body("")?;

        let response = ServiceBuilder::new()
            .layer(HttpClientLayer)
            .service(client.clone())
            .call(request.clone())
            .await?;
        assert!(response.status().is_success());
        // Try to read body
        let info = Info::from_body(response.into_body()).await?;
        assert!(info.request_id.is_none());

        // TODO Find the way to avoid cloning the service.
        let service = ServiceBuilder::new()
            .override_response_header(USER_AGENT, HeaderValue::from_static("tower-reqwest"))
            .set_x_request_id(MakeRequestUuid)
            .layer(HttpClientLayer)
            .service(client)
            .boxed_clone();
        // Execute request with a several layers from the tower-http
        let response = service.clone().call(request).await?;

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
