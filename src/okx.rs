use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::{GET_BALANCE_PATH, LOAD_MARKETS_PATH, OKX_BASE_DOMAIN_URL};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, NaiveDateTime, SecondsFormat, Utc};
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
    ) -> eyre::Result<()> {
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
        println!("{value:#?}");

        Ok(())
    }

    pub async fn fetch_balance(&self, ccy: Option<Vec<String>>) {
        let params = ccy
            .map(|ccy| ccy.join(","))
            .map(|ccies| vec![("ccy".to_string(), ccies)]);

        let _ = self
            .send_request(GET_BALANCE_PATH, params, None, reqwest::Method::GET, None)
            .await;
    }

    pub async fn load_markets(&self) {
        let _ = self
            .send_request(LOAD_MARKETS_PATH, None, None, reqwest::Method::GET, None)
            .await;
    }
}
