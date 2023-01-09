pub struct GetOAuthTokenRequest {
    consumer_key: String,
    redirect_uri: String,
    state: Option<String>,
}

pub struct GetOAuthTokenResponse {
    code: String,
    state: Option<String>,
}

pub struct GetAccessTokenRequest {
    consumer_key: String,
    code: String,
}

pub struct GetAccessTokenResponse {
    access_token: String,
    username: String,
}
