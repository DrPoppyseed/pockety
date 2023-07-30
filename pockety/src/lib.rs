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
use reqwest::Client;
use serde::{self, de::DeserializeOwned, Deserialize, Serialize};
pub mod api;
mod error;
pub use error::{ApiError, Error, HttpError};
pub mod models;
pub use reqwest;
use url::Url;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PocketyUrl(Inner);

#[derive(Clone, PartialEq, Eq, Debug)]
enum Inner {
    Base,
    Authorize,
}

impl PocketyUrl {
    pub const BASE: PocketyUrl = PocketyUrl(Inner::Base);

    pub const AUTHORIZE: PocketyUrl = PocketyUrl(Inner::Authorize);

    pub fn as_str(&self) -> &str {
        match self.0 {
            Inner::Base => "https://getpocket.com/v3",
            Inner::Authorize => "https://getpocket.com/auth/authorize",
        }
    }

    pub fn as_url(&self) -> Url {
        Url::parse(self.as_str()).unwrap()
    }
}

impl Default for PocketyUrl {
    fn default() -> Self {
        Self::BASE
    }
}

impl std::fmt::Display for PocketyUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GetAccessTokenResponse {
    pub access_token: String,
    pub username: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Pockety {
    pub base_url: Url,
    pub redirect_url: Url,
    pub(crate) consumer_key: String,
    pub client: Client,
}

impl Pockety {
    pub fn new(consumer_key: String, redirect_url: &str) -> Self {
        Self {
            base_url: PocketyUrl::BASE.as_url(),
            redirect_url: Url::parse(redirect_url).unwrap(),
            consumer_key,
            client: Client::new(),
        }
    }

    pub async fn post<Body, Res>(
        &self,
        relative_url: &str,
        body: Option<&Body>,
    ) -> Result<Res, Error>
    where
        Body: Serialize,
        Res: DeserializeOwned,
    {
        let url = format!("{base_url}{relative_url}", base_url = self.base_url);

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
                    let res = response.json().await?;
                    Ok(res)
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

    // TODO: Type from String to impl Into<String>
    pub async fn get_access_token(
        &self,
        request_token: String,
        state: Option<String>,
    ) -> Result<GetAccessTokenResponse, Error> {
        self.post(
            "/oauth/authorize",
            Some(&GetAccessTokenRequest {
                consumer_key: self.consumer_key.clone(),
                code: request_token,
                state,
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
