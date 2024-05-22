use crate::okx::Okx;

use dotenv::dotenv;

#[tokio::test]
async fn test_fetch_balance_all_ccies() {
    dotenv().ok();
    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let exchange = Okx::new(secret_key, api_key, password);

    exchange.get_trading_account_balance(None).await.unwrap();
}

#[tokio::test]
async fn test_fetch_balance_specific_ccies() {
    dotenv().ok();

    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let exchange = Okx::new(secret_key, api_key, password);
    let ccies = Some(vec!["USDC".to_string()]);

    exchange.get_trading_account_balance(ccies).await.unwrap();
}

#[tokio::test]
async fn test_fetch_trading_balance_sub_accounts() {
    dotenv().ok();

    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");
    let account_name = std::env::var("SUB_ACCOUNT_NAME").expect("SUB_ACCOUNT_NAME must be set");
    let exchange = Okx::new(secret_key, api_key, password);
    exchange
        .get_sub_account_funding_balance(account_name.as_str())
        .await
        .unwrap();
}

#[tokio::test]
async fn test_fetch_sub_accounts_list() {
    dotenv().ok();

    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let exchange = Okx::new(secret_key, api_key, password);
    exchange.get_sub_accounts_list().await.unwrap();
}

#[tokio::test]
async fn test_get_all_sub_accounts_funding_balances() {
    dotenv().ok();

    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let exchange = Okx::new(secret_key, api_key, password);
    exchange.get_sub_accounts_funding_balances().await.unwrap();
}

#[tokio::test]
async fn test_transfer_from_sub_accounts() {
    dotenv().ok();

    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let exchange = Okx::new(secret_key, api_key, password);

    let ccy = "ETH".to_string();
    exchange
        .transfer_assets_from_sub_accounts_to_master(ccy)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_withdrawal() {
    dotenv().ok();

    let password = std::env::var("API_PASSWORD").expect("API_PASSWORD must be set");
    let secret_key = std::env::var("API_SECRET").expect("API_SECRET must be set");
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set");

    let exchange = Okx::new(secret_key, api_key, password);

    let ccy = "SOL".to_string();
    let address = "".to_string();
    let chain = "Solana".to_string();
    let fee = "0.008".to_string();
    let amount = 0.01;
    exchange
        .withdraw(amount, fee, ccy, chain, address)
        .await
        .unwrap();
}
