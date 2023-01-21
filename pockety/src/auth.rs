use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct GetRequestTokenRequest {
    pub consumer_key: String,
    pub redirect_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GetRequestTokenResponse {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GetAccessTokenRequest {
    pub consumer_key: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GetAccessTokenResponse {
    pub access_token: String,
    pub username: String,
    pub state: Option<String>,
}
