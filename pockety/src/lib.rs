#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use std::str::FromStr;

use api::{add::AddHandler, modify::ModifyHandler, retrieve::RetrieveHandler};
use futures::TryFutureExt;
use reqwest::{header::HeaderMap, Client};
use serde::{self, de::DeserializeOwned, Deserialize, Serialize};
pub mod api;
mod error;
pub use error::{ApiError, Error, HttpError};
pub mod models;
pub use reqwest;

#[derive(Serialize, Debug, Clone)]
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

#[derive(Serialize, Debug, Clone)]
pub struct GetAccessTokenRequest {
    pub consumer_key: String,
    pub code: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetAccessTokenResponse {
    pub access_token: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
pub struct RateLimits {
    /// X-Limit-User-Limit: Current rate limit enforced per user
    pub user_limit: Option<u32>,
    /// X-Limit-User-Remaining: Number of calls remaining before hitting user's rate limit
    pub user_remaining: Option<u32>,
    /// X-Limit-User-Reset: Seconds until user's rate limit resets
    pub user_reset: Option<u32>,
    /// X-Limit-Key-Limit: Current rate limit enforced per consumer key
    pub key_limit: Option<u32>,
    /// X-Limit-Key-Remaining: Number of calls remaining before hitting consumer key's rate limit
    pub key_remaining: Option<u32>,
    /// X-Limit-Key-Reset: Seconds until consumer key rate limit resets
    pub key_reset: Option<u32>,
}

impl RateLimits {
    pub const USER_LIMIT_HEADER: &str = "X-Limit-User-Limit";
    pub const USER_REMAINING_HEADER: &str = "X-Limit-User-Remaining";
    pub const USER_RESET_HEADER: &str = "X-Limit-User-Reset";
    pub const KEY_LIMIT_HEADER: &str = "X-Limit-Key-Limit";
    pub const KEY_REMAINING_HEADER: &str = "X-Limit-Key-Remaining";
    pub const KEY_RESET_HEADER: &str = "X-Limit-Key-Reset";

    pub fn from_headers(headers: &HeaderMap) -> Self {
        Self {
            user_limit: get_header(headers, Self::USER_LIMIT_HEADER),
            user_remaining: get_header(headers, Self::USER_REMAINING_HEADER),
            user_reset: get_header(headers, Self::USER_RESET_HEADER),
            key_limit: get_header(headers, Self::KEY_LIMIT_HEADER),
            key_remaining: get_header(headers, Self::KEY_REMAINING_HEADER),
            key_reset: get_header(headers, Self::KEY_RESET_HEADER),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PocketyResponse<T> {
    pub rate_limits: RateLimits,
    pub data: T,
}

pub type ApiResult<T> = Result<PocketyResponse<T>, Error>;

#[derive(Debug, Clone)]
pub struct Pockety {
    pub base_url: String,
    pub redirect_url: String,
    pub(crate) consumer_key: String,
    pub client: Client,
}

fn get_header<T>(headers: &HeaderMap, header: &str) -> Option<T>
where
    T: FromStr,
{
    headers
        .get(header)
        .and_then(|value| value.to_str().ok().and_then(|v| str::parse::<T>(v).ok()))
}

impl Pockety {
    pub const BASE_URL: &str = "https://getpocket.com/v3";
    pub const AUTHORIZE_URL: &str = "https://getpocket.com/auth/authorize";

    pub fn new<T, U>(consumer_key: T, redirect_url: U) -> Result<Self, Error>
    where
        T: Into<String>,
        U: Into<String>,
    {
        let pockety = Self {
            base_url: Self::BASE_URL.to_string(),
            redirect_url: redirect_url.into(),
            consumer_key: consumer_key.into(),
            client: Client::new(),
        };

        Ok(pockety)
    }

    pub async fn post<T, U>(&self, relative_url: &str, body: Option<&T>) -> ApiResult<U>
    where
        T: Serialize,
        U: DeserializeOwned,
    {
        let url = format!("{}{relative_url}", self.base_url);

        let request = self
            .client
            .post(url)
            .header("X-Accept", "application/json")
            .header("Content-Type", "application/json; charset=UTF-8");

        let request = if let Some(body) = body {
            request.json(body)
        } else {
            request
        }
        .build()?;

        match self.client.execute(request).await {
            Ok(response) => {
                let headers = response.headers();
                let rate_limits = RateLimits::from_headers(headers);

                if response.status().is_success() {
                    response
                        .json::<U>()
                        .map_err(|e| Error::Parse(e.to_string()))
                        .map_ok(|data| PocketyResponse { rate_limits, data })
                        .await
                } else {
                    let mut http_error = HttpError::new().status_code(response.status());
                    http_error.error_code = get_header(headers, "X-Error-Code");
                    http_error.error_message = get_header(headers, "X-Error");
                    Err(Error::Http(http_error))
                }
            }
            Err(e) => Err(Error::Http(HttpError::new().error_message(e))),
        }
    }

    pub async fn get_request_token(
        &self,
        state: Option<String>,
    ) -> ApiResult<GetRequestTokenResponse> {
        self.post(
            "/oauth/request",
            Some(&GetRequestTokenRequest {
                consumer_key: self.consumer_key.clone(),
                redirect_uri: self.redirect_url.to_string(),
                state,
            }),
        )
        .await
    }

    pub async fn get_access_token(
        &self,
        request_token: impl Into<String>,
    ) -> ApiResult<GetAccessTokenResponse> {
        self.post(
            "/oauth/authorize",
            Some(&GetAccessTokenRequest {
                consumer_key: self.consumer_key.clone(),
                code: request_token.into(),
            }),
        )
        .await
    }

    pub fn retrieve(&self) -> RetrieveHandler {
        RetrieveHandler::new(self)
    }

    pub fn modify(&self) -> ModifyHandler {
        ModifyHandler::new(self)
    }

    pub fn add(&self) -> AddHandler {
        AddHandler::new(self)
    }
}
