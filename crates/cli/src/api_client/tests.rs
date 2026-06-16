use super::client::ApiClient;
use reqwest::Client;

#[test]
fn test_api_client_creation() {
    let http_client = match Client::builder().build() {
        Ok(c) => c,
        Err(_) => return,
    };
    let client = ApiClient {
        client: http_client,
        base_url: "http://localhost:3000".to_string(),
        api_key: "test_key".to_string(),
    };

    assert_eq!(client.base_url, "http://localhost:3000");
    assert_eq!(client.api_key, "test_key");
}
