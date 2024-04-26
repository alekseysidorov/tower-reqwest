use http_body_util::Full;
use serde_json::{json, Value};
use tower_http_client::body_reader::BodyReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_vec(&json!({ "id": 1234 })).unwrap();
    let body = Full::new(data.as_ref());
    let content: Value = BodyReader::new(body).json().await?;

    assert_eq!(content["id"], 1234);
    Ok(())
}
