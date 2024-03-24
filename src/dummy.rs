use futures_util::{future::BoxFuture, FutureExt};
use http::{header::USER_AGENT, HeaderValue};
use tower::{Layer, Service};

pub struct DummyLayer;

pub struct DummyService<S> {
    inner: S,
}

impl<S> Layer<S> for DummyLayer {
    type Service = DummyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DummyService { inner }
    }
}

impl<S> Service<reqwest::Request> for DummyService<S>
where
    S: Service<reqwest::Request, Response = reqwest::Response>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: reqwest::Request) -> Self::Future {
        println!("Sending request to {}", req.url());

        let fut = self.inner.call(req);
        async move {
            let mut resp = fut.await?;
            resp.headers_mut()
                .insert(USER_AGENT, HeaderValue::from_static("tower-reqwest"));
            Ok(resp)
        }
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use http::{header::USER_AGENT, HeaderValue};
    use pretty_assertions::assert_eq;
    use reqwest::{Client, Request, Url};
    use tower::{Service, ServiceBuilder};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::dummy::DummyLayer;

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
        // Create reqwest client.
        let mut client = Client::new();

        // Execute request without layers
        let response = client
            .call(Request::new(
                reqwest::Method::GET,
                Url::parse(&format!("{mock_uri}/hello"))?,
            ))
            .await?;
        assert!(response.status().is_success());
        // Execute request via ServiceBuilder
        let mut service = ServiceBuilder::new().layer(DummyLayer).service(&client);
        let response = service
            .call(Request::new(
                reqwest::Method::GET,
                Url::parse(&format!("{mock_uri}/hello"))?,
            ))
            .await?;

        dbg!(&response);
        assert!(response.status().is_success());
        assert_eq!(
            response.headers().get(USER_AGENT).unwrap(),
            HeaderValue::from_static("tower-reqwest")
        );

        Ok(())
    }
}
