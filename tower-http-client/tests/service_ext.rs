use http::{header::USER_AGENT, HeaderValue};
use reqwest::Client;
use tower::ServiceBuilder;
use tower_http::ServiceBuilderExt;
use tower_http_client::ServiceExt as _;
use tower_reqwest::HttpClientLayer;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

mod utils;

// Check that we can use tower-http layers on top of the compatibility wrapper.
#[tokio::test]
async fn test_service_ext_execute() -> anyhow::Result<()> {
    let (mock_server, mock_uri) = utils::start_mock_server().await;
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

// Check that the `get` method is useful.
#[tokio::test]
async fn test_service_ext_get() -> anyhow::Result<()> {
    let (mock_server, mock_uri) = utils::start_mock_server().await;
    // Arrange the behaviour of the MockServer adding a Mock:
    // when it receives a GET request on '/hello' it will respond with a 200.
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200))
        // Mounting the mock on the mock server - it's now effective!
        .mount(&mock_server)
        .await;

    let client = ServiceBuilder::new()
        .layer(HttpClientLayer)
        .service(Client::new());

    let response = client.get(format!("{mock_uri}/hello")).send()?.await?;
    assert!(response.status().is_success());

    Ok(())
}
