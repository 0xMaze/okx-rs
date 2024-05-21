use crate::okx::Okx;

use dotenv::dotenv;

#[tokio::test]
async fn test_fetch_balance_all_ccies() {
    dotenv().ok();
    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let exchange = Okx::new(secret_key, api_key, password);

    exchange.fetch_balance(None).await;
}

#[tokio::test]
async fn test_fetch_balance_specific_ccies() {
    dotenv().ok();

    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let exchange = Okx::new(secret_key, api_key, password);
    let ccies = Some(vec!["ETH".to_string()]);
    exchange.fetch_balance(ccies).await;
}
