//! Adapter for [`reqwest-middleware`] client.
//!
//! [`reqwest-middleware`]: https://crates.io/crates/reqwest-middleware

use std::{
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::{future::BoxFuture, FutureExt};
use reqwest::Client;
use reqwest_middleware::{ClientWithMiddleware, Middleware};
use tower::Service;

use crate::HttpClientService;

impl From<ClientWithMiddleware> for HttpClientService<ClientWithMiddleware> {
    fn from(value: ClientWithMiddleware) -> Self {
        Self(value)
    }
}

impl From<(Client, Vec<Arc<dyn Middleware>>)> for HttpClientService<ClientWithMiddleware> {
    fn from((client, middleware_stack): (Client, Vec<Arc<dyn Middleware>>)) -> Self {
        Self::from(ClientWithMiddleware::new(client, middleware_stack))
    }
}

impl Service<reqwest::Request> for HttpClientService<ClientWithMiddleware> {
    type Response = reqwest::Response;
    type Error = reqwest_middleware::Error;
    // TODO We need ATPIT to get rid of boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: reqwest::Request) -> Self::Future {
        let client = self.0.clone();
        async move { client.execute(req).await }.boxed()
    }
}

impl Service<reqwest::Request> for &HttpClientService<ClientWithMiddleware> {
    type Response = reqwest::Response;
    type Error = reqwest_middleware::Error;
    // TODO We need ATPIT to get rid of boxing.
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: reqwest::Request) -> Self::Future {
        let client = self.0.clone();
        async move { client.execute(req).await }.boxed()
    }
}

impl From<reqwest_middleware::Error> for crate::error::Error {
    fn from(value: reqwest_middleware::Error) -> Self {
        match value {
            reqwest_middleware::Error::Middleware(err) => Self::Middleware(err.into()),
            reqwest_middleware::Error::Reqwest(err) => Self::Client(err.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use http::{header::USER_AGENT, HeaderValue};
    use reqwest::Client;
    use reqwest_middleware::ClientWithMiddleware;
    use tower::{Service, ServiceBuilder};
    use tower_http::ServiceBuilderExt;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{HttpClientLayer, HttpClientService};

    // Check that we can use tower-http layers on top of the compatibility wrapper.
    #[tokio::test]
    async fn test_reqwest_middleware_layer_simple() -> anyhow::Result<()> {
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

        let client = ClientWithMiddleware::new(Client::new(), []);
        let response = ServiceBuilder::new()
            .override_response_header(USER_AGENT, HeaderValue::from_static("tower-reqwest"))
            .layer(HttpClientLayer)
            .service(HttpClientService::from(client))
            .call(
                http::request::Builder::new()
                    .method(http::Method::GET)
                    .uri(format!("{mock_uri}/hello"))
                    // TODO Make in easy to create requests without body.
                    .body(http_body_util::Empty::new())?,
            )
            .await?;

        assert!(response.status().is_success());
        assert_eq!(
            response.headers().get(USER_AGENT).unwrap(),
            HeaderValue::from_static("tower-reqwest")
        );

        Ok(())
    }
}
