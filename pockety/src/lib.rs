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

use api::{add::AddHandler, modify::ModifyHandler, retrieve::RetrieveHandler};
use futures::TryFutureExt;
use reqwest::Client;
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

#[derive(Debug, Clone)]
pub struct Pockety {
    pub base_url: String,
    pub redirect_url: String,
    pub(crate) consumer_key: String,
    pub client: Client,
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

    pub async fn post<T, U>(
        &self,
        relative_url: &str,
        body: Option<&T>,
    ) -> Result<U, Error>
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
                if response.status().is_success() {
                    response
                        .json::<serde_json::Value>()
                        .map_err(Error::from)
                        .and_then(|json| async {
                            serde_json::from_value::<U>(json)
                                .map_err(|e| Error::Parse(e.to_string()))
                        })
                        .await
                } else {
                    let mut http_error =
                        HttpError::new().status_code(response.status());
                    http_error.error_code = response
                        .headers()
                        .get("X-Error-Code")
                        .map(|v| v.to_str().map(|v| v.to_string()))
                        .transpose()?;
                    http_error.error_message = response
                        .headers()
                        .get("X-Error")
                        .map(|v| v.to_str().map(|v| v.to_string()))
                        .transpose()?;
                    Err(Error::Http(http_error))
                }
            }
            Err(e) => Err(Error::Http(
                HttpError::new().error_message(Some(e.to_string())),
            )),
        }
    }

    pub async fn get_request_token(
        &self,
        state: Option<String>,
    ) -> Result<GetRequestTokenResponse, Error> {
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
    ) -> Result<GetAccessTokenResponse, Error> {
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
