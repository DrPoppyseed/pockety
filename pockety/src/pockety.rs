use std::sync::Arc;

use reqwest::{Client, StatusCode, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{
    api::{add::AddHandler, modify::ModifyHandler, retrieve::RetrieveHandler},
    error::{Error, HttpError},
};

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

#[derive(Debug, Default, Clone)]
pub struct Auth {
    pub(crate) consumer_key: String,
    pub(crate) access_token: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone)]
pub struct Pockety {
    pub base_url: Url,
    pub redirect_url: Url,
    pub(crate) auth: Auth,
    pub client: Client,
}

impl Pockety {
    pub fn new(consumer_key: &str, redirect_url: &str) -> Self {
        let auth = Auth {
            consumer_key: consumer_key.to_string(),
            access_token: Arc::new(Mutex::new(None)),
        };

        Self {
            base_url: PocketyUrl::BASE.as_url(),
            redirect_url: Url::parse(redirect_url).unwrap(),
            auth,
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

        if let Ok(response) = self.client.execute(request).await {
            if response.status().is_success() {
                let res = response.json().await?;
                Ok(res)
            } else {
                let mut http_error = HttpError::new(response.status());
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
        } else {
            Err(Error::Http(HttpError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
            )))
        }
    }

    pub async fn get_request_token(
        &self,
        state: Option<String>,
    ) -> Result<String, Error> {
        let body = GetRequestTokenRequest {
            consumer_key: self.auth.consumer_key.clone(),
            redirect_uri: self.redirect_url.to_string(),
            state,
        };

        let response = self
            .post::<_, GetRequestTokenResponse>("/oauth/request", Some(&body))
            .await?;

        // TODO: consider using domain primitives instead of strings
        Ok(response.code)
    }

    pub async fn get_access_token(
        &self,
        request_token: &str,
        state: Option<String>,
    ) -> Result<String, Error> {
        let body = GetAccessTokenRequest {
            consumer_key: self.auth.consumer_key.clone(),
            code: request_token.to_string(),
            state,
        };

        let response = self
            .post::<_, GetAccessTokenResponse>("/oauth/authorize", Some(&body))
            .await?;

        let mut guard = self.auth.access_token.lock().await;
        *guard = Some(response.access_token.clone());

        Ok(response.access_token)
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
