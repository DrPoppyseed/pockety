use reqwest::{Client, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    api::{add::AddHandler, modify::ModifyHandler, retrieve::RetrieveHandler},
    error::{HttpError, PocketyError, PocketyError::Http},
};

static POCKET_BASE_URL: &str = "https://getpocket.com/v3/";

#[derive(Debug, Default, Clone)]
pub struct Auth {
    pub(crate) consumer_key:  String,
    pub(crate) request_token: Option<String>,
    pub(crate) access_token:  Option<String>,
}

#[derive(Debug, Clone)]
pub struct Pockety {
    pub base_url:    Url,
    pub(crate) auth: Auth,
    pub client:      Client,
}

impl Pockety {
    pub fn new(consumer_key: &str) -> Self {
        let auth = Auth {
            consumer_key:  consumer_key.to_string(),
            request_token: None,
            access_token:  None,
        };

        Self {
            base_url: Url::parse(POCKET_BASE_URL).unwrap(),
            auth,
            client: Client::new(),
        }
    }

    pub async fn post<Body, Res>(
        &self,
        relative_url: &str,
        body: Option<&Body>,
    ) -> Result<Res, PocketyError>
    where
        Body: Serialize,
        Res: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, relative_url);

        let mut request = self.client.post(url);
        if let Some(body) = body {
            request = request.json(body);
        }
        let request = request.build()?;

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
                Err(Http(http_error))
            }
        } else {
            Err(Http(HttpError::new(StatusCode::INTERNAL_SERVER_ERROR)))
        }
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
