#[derive(Debug)]
pub struct GetOAuthTokenRequest {
    pub consumer_key: String,
    pub redirect_uri: String,
    pub state:        Option<String>,
}

#[derive(Debug)]
pub struct GetOAuthTokenResponse {
    pub code:  String,
    pub state: Option<String>,
}

#[derive(Debug)]
pub struct GetAccessTokenRequest {
    pub consumer_key: String,
    pub code:         String,
}

#[derive(Debug)]
pub struct GetAccessTokenResponse {
    pub access_token: String,
    pub username:     String,
}
