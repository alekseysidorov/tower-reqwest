use wiremock::MockServer;

// Starts a background HTTP server on a random local port and returns
// a tuple with the mock server and its base uri.
pub async fn start_mock_server() -> (MockServer, String) {
    let mock_server = MockServer::start().await;
    let mock_uri = mock_server.uri();
    (mock_server, mock_uri)
}
