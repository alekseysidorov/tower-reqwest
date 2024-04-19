//! Various utilities and extensions for working with Tower http clients.

use std::future::Future;

use tower::Service;

use crate::{HttpBody, HttpResponse};

/// An extension trait for Tower HTTP services with the typical client methods.
pub trait HttpClientExt: Clone {
    /// Executes an HTTP request.
    fn execute<B>(
        &self,
        request: http::Request<B>,
    ) -> impl Future<Output = crate::Result<HttpResponse>>
    where
        B: Into<HttpBody>;
}

impl<S> HttpClientExt for S
where
    S: Service<http::Request<HttpBody>, Response = HttpResponse, Error = crate::Error>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: 'static,
{
    fn execute<B>(
        &self,
        request: http::Request<B>,
    ) -> impl Future<Output = crate::Result<HttpResponse>>
    where
        B: Into<HttpBody>,
    {
        let request = request.map(Into::into);
        self.clone().call(request)
    }
}

#[cfg(test)]
mod tests {
    use http::{header::USER_AGENT, HeaderValue};
    use reqwest::Client;
    use tower::ServiceBuilder;
    use tower_http::ServiceBuilderExt;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{util::HttpClientExt, HttpClientLayer};

    // Check that we can use tower-http layers on top of the compatibility wrapper.
    #[tokio::test]
    async fn test_reqwest_http_client_util() -> anyhow::Result<()> {
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

        let client = ServiceBuilder::new()
            .override_response_header(USER_AGENT, HeaderValue::from_static("tower-reqwest"))
            .layer(HttpClientLayer)
            .service(Client::new());
        let response = client
            .execute(
                http::request::Builder::new()
                    .method(http::Method::GET)
                    .uri(format!("{mock_uri}/hello"))
                    // TODO Make in easy to create requests without body.
                    .body("")?,
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