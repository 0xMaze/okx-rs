use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    constants::{
        ASSETS_TRANSFER_PATH, ASSETS_WITHDRAWAL_PATH, GET_SUB_ACCOUNTS_FUNDING_BALANCE_PATH,
        GET_SUB_ACCOUNTS_LIST_PATH, GET_TRADING_ACCOUNT_BALANCE_PATH, OKX_BASE_DOMAIN_URL,
    },
    schemas::{
        AssetWithdrawalSchema, AssetsTransferSchema, AssetsTrasnferData, AssetsWithdrawalData,
        GetBalanceResponseDataDetails, GetSubAccountListData, GetTradingBalanceResponseData,
        OkxResponseSchema,
    },
};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
use eyre::eyre;
use hmac::{Hmac, Mac};
use reqwest::{header::HeaderMap, Proxy, Url};
use sha2::Sha256;

#[derive(Debug)]
pub struct Credentials {
    secret_key: String,
    api_key: String,
    password: String,
}

impl Credentials {
    fn new(secret_key: String, api_key: String, password: String) -> Self {
        Self {
            secret_key,
            api_key,
            password,
        }
    }

    fn get_signature(
        &self,
        method: reqwest::Method,
        timestamp: String,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> String {
        let get_request_bytes = |timestamp: String,
                                 method: reqwest::Method,
                                 path: &str,
                                 body: Option<serde_json::Value>| {
            let body_string = if let Some(body) = body {
                serde_json::to_string(&body).unwrap()
            } else {
                "".to_string()
            };

            let request_string = format!("{}{}{}{}", timestamp, method, path, body_string);
            request_string.into_bytes()
        };

        let mac_bytes = {
            let secret_bytes = self.secret_key.as_bytes();
            let request_bytes = get_request_bytes(timestamp, method, path, body);

            let mut mac = Hmac::<Sha256>::new_from_slice(secret_bytes).unwrap();
            mac.update(request_bytes.as_slice());
            mac.finalize().into_bytes().to_vec()
        };

        general_purpose::STANDARD.encode(mac_bytes)
    }
}

pub struct Okx {
    credentials: Credentials,
}

impl Okx {
    pub fn new(secret_key: String, api_key: String, password: String) -> Self {
        Self {
            credentials: Credentials::new(secret_key, api_key, password),
        }
    }

    #[allow(unused)]
    fn milliseconds() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    #[allow(deprecated, unused)]
    fn iso8601(timestamp: u128) -> String {
        let x = timestamp as f64;

        let nt = NaiveDateTime::from_timestamp(
            (x / 1000.0).floor() as i64,
            ((x * 1e6).floor() as u64 % 1e9 as u64) as u32,
        );
        let t: DateTime<Utc> = DateTime::from_utc(nt, Utc);
        t.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    pub fn build_url(path: &str, params: Option<Vec<(String, String)>>) -> Url {
        let base_url = format!("{}{}", OKX_BASE_DOMAIN_URL, path);
        if let Some(params) = params {
            Url::parse_with_params(&base_url, params).unwrap()
        } else {
            Url::parse(&base_url).unwrap()
        }
    }

    fn extract_path_and_params(url: &Url) -> String {
        let path = url.path();
        if let Some(query) = url.query() {
            format!("{path}?{query}")
        } else {
            path.to_string()
        }
    }

    fn get_auth_headers(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> HeaderMap {
        let formatted_timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let signature =
            self.credentials
                .get_signature(method, formatted_timestamp.clone(), path, body);

        let headers = [
            ("OK-ACCESS-KEY", self.credentials.api_key.as_str()), // The API Key as a String.
            ("OK-ACCESS-PASSPHRASE", self.credentials.password.as_str()), // The passphrase you specified when creating the APIKey.
            ("OK-ACCESS-SIGN", signature.as_str()), // The Base64-encoded signature (see Signing Messages subsection for details).
            ("OK-ACCESS-TIMESTAMP", formatted_timestamp.as_str()), // The UTC timestamp of your request .e.g : 2020-12-08T09:08:57.715Z
        ];

        let mut map = HeaderMap::with_capacity(headers.len());
        headers.into_iter().for_each(|(key, value)| {
            map.insert(key, reqwest::header::HeaderValue::from_str(value).unwrap());
        });

        map
    }

    async fn send_request(
        &self,
        path: &str,
        params: Option<Vec<(String, String)>>,
        proxy: Option<&str>,
        method: reqwest::Method,
        body: Option<serde_json::Value>,
    ) -> eyre::Result<serde_json::Value> {
        let client_builder = reqwest::ClientBuilder::new();
        let client = if let Some(proxy) = proxy {
            client_builder.proxy(Proxy::all(proxy)?).build()
        } else {
            client_builder.build()
        }?;

        let url = Self::build_url(path, params);
        let path_and_params = Self::extract_path_and_params(&url);
        let headers = self.get_auth_headers(method.clone(), &path_and_params, body.clone());

        let request_builder = client.request(method, url).headers(headers);

        let request = if let Some(body) = body {
            request_builder.json(&body).build()?
        } else {
            request_builder.build()?
        };

        let response = client.execute(request).await?;
        let value = response.json::<serde_json::Value>().await?;

        Ok(value)
    }

    pub async fn get_trading_account_balance(
        &self,
        ccy: Option<Vec<String>>,
    ) -> eyre::Result<OkxResponseSchema<GetTradingBalanceResponseData>> {
        let params = ccy
            .map(|ccy| ccy.join(","))
            .map(|ccies| vec![("ccy".to_string(), ccies)]);

        let response_body = self
            .send_request(
                GET_TRADING_ACCOUNT_BALANCE_PATH,
                params,
                None,
                reqwest::Method::GET,
                None,
            )
            .await?;

        let balance_schema = serde_json::from_value::<
            OkxResponseSchema<GetTradingBalanceResponseData>,
        >(response_body)?;

        Ok(balance_schema)
    }

    pub async fn get_sub_account_funding_balance(
        &self,
        account_name: &str,
    ) -> eyre::Result<OkxResponseSchema<GetBalanceResponseDataDetails>> {
        let params = vec![("subAcct".to_string(), account_name.to_string())];

        let response_body = self
            .send_request(
                GET_SUB_ACCOUNTS_FUNDING_BALANCE_PATH,
                Some(params),
                None,
                reqwest::Method::GET,
                None,
            )
            .await?;

        let balance_schema = serde_json::from_value::<
            OkxResponseSchema<GetBalanceResponseDataDetails>,
        >(response_body)?;

        Ok(balance_schema)
    }

    pub async fn get_sub_accounts_list(
        &self,
    ) -> eyre::Result<OkxResponseSchema<GetSubAccountListData>> {
        let response_body = self
            .send_request(
                GET_SUB_ACCOUNTS_LIST_PATH,
                None,
                None,
                reqwest::Method::GET,
                None,
            )
            .await?;

        let sub_account_list_schema =
            serde_json::from_value::<OkxResponseSchema<GetSubAccountListData>>(response_body)?;
        Ok(sub_account_list_schema)
    }

    pub async fn get_sub_accounts_funding_balances(
        &self,
    ) -> eyre::Result<HashMap<String, OkxResponseSchema<GetBalanceResponseDataDetails>>> {
        let accounts = self.get_sub_accounts_list().await?;
        let mut balances_map = HashMap::with_capacity(accounts.data.len());

        for account in accounts.data {
            let account_name = account.sub_acct;
            let balance_schema = self.get_sub_account_funding_balance(&account_name).await?;
            balances_map.insert(account_name, balance_schema);
        }

        Ok(balances_map)
    }

    async fn transfer_assets_from_sub_account_to_master(
        &self,
        ccy: &str,
        amount: String,
        sub_acct_name: String,
    ) -> eyre::Result<()> {
        let body = AssetsTransferSchema::new(ccy.to_string(), amount, sub_acct_name);
        let response_body = self
            .send_request(
                ASSETS_TRANSFER_PATH,
                None,
                None,
                reqwest::Method::POST,
                Some(serde_json::to_value(body).unwrap()),
            )
            .await?;

        let body = serde_json::from_value::<OkxResponseSchema<AssetsTrasnferData>>(response_body)?;

        if body.code == "0" {
            Ok(())
        } else {
            Err(eyre!("Response code is not OK: {}", body.code))
        }
    }

    pub async fn transfer_assets_from_sub_accounts_to_master(
        &self,
        ccy: String,
    ) -> eyre::Result<()> {
        let balances_map = self.get_sub_accounts_funding_balances().await?;

        for (account_name, balance_schema) in balances_map {
            if let Some(data) = balance_schema.get_data() {
                if let Some(balance) = data.get_balance(&ccy) {
                    self.transfer_assets_from_sub_account_to_master(&ccy, balance, account_name)
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn withdraw(
        &self,
        amt: f64,
        fee: String,
        ccy: String,
        chain: String,
        to_addr: String,
    ) -> eyre::Result<()> {
        let body = AssetWithdrawalSchema::new(amt, fee, ccy, chain, to_addr);
        let response_body = self
            .send_request(
                ASSETS_WITHDRAWAL_PATH,
                None,
                None,
                reqwest::Method::POST,
                Some(serde_json::to_value(body).unwrap()),
            )
            .await?;

        let body =
            serde_json::from_value::<OkxResponseSchema<AssetsWithdrawalData>>(response_body)?;

        if body.code == "0" {
            Ok(())
        } else {
            Err(eyre!("Response code is not OK: {}", body.code))
        }
    }
}
