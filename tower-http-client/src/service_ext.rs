use std::future::Future;

use tower::Service;

/// An extension trait for Tower HTTP services with the typical client methods.
pub trait ServiceExt<ReqBody, RespBody, Err> {
    /// Executes an HTTP request.
    fn execute<R>(
        &self,
        request: http::Request<R>,
    ) -> impl Future<Output = Result<http::Response<RespBody>, Err>>
    where
        ReqBody: From<R>;
}

impl<S, ReqBody, RespBody, Err> ServiceExt<ReqBody, RespBody, Err> for S
where
    S: Service<http::Request<ReqBody>, Response = http::Response<RespBody>, Error = Err>
        + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
    S::Error: 'static,
{
    fn execute<R>(
        &self,
        request: http::Request<R>,
    ) -> impl Future<Output = Result<http::Response<RespBody>, Err>>
    where
        ReqBody: From<R>,
    {
        self.clone().call(request.map(ReqBody::from))
    }
}

#[cfg(test)]
mod tests {
    use http::{header::USER_AGENT, HeaderValue};
    use reqwest::Client;
    use tower::ServiceBuilder;
    use tower_http::ServiceBuilderExt;
    use tower_reqwest::HttpClientLayer;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::ServiceExt;

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
