use http_body_util::Full;
use tower_http_client::body_reader::BodyReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let body = Full::new("Hello world".as_bytes());
    let content = BodyReader::new(body).bytes().await?;

    assert_eq!(content, "Hello world");
    Ok(())
}
