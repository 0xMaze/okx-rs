use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct OkxResponseSchema<T> {
    pub code: String,
    pub data: Vec<T>,
}

impl<T> OkxResponseSchema<T> {
    pub fn get_data(&self) -> Option<&T> {
        self.data.first().or(None)
    }
}

#[derive(Deserialize, Debug)]
pub struct GetTradingBalanceResponseData {
    pub details: Vec<GetBalanceResponseDataDetails>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetBalanceResponseDataDetails {
    pub avail_bal: String,
    pub ccy: String,
}

impl GetBalanceResponseDataDetails {
    pub fn get_balance(&self, ccy: &str) -> Option<String> {
        (self.ccy == ccy).then(|| self.avail_bal.clone())
    }
}

impl GetTradingBalanceResponseData {
    pub fn get_balance(&self, ccy: &str) -> Option<String> {
        self.details
            .iter()
            .find(|detail| detail.ccy == ccy)
            .map(|detail| detail.avail_bal.to_string())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSubAccountListData {
    pub sub_acct: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssetsTransferSchema {
    ccy: String,
    amt: String,
    from: String,
    to: String,
    #[serde(rename = "type")]
    _type: String,
    sub_acct: String,
}

impl AssetsTransferSchema {
    pub fn new(ccy: String, amt: String, sub_acct: String) -> Self {
        Self {
            ccy,
            amt,
            from: "6".to_string(),
            to: "6".to_string(),
            _type: "2".to_string(),
            sub_acct,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetsTrasnferData {
    trans_id: String,
    ccy: String,
    amt: String,
}
