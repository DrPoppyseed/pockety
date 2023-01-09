use futures::TryFutureExt;
use reqwest::{Client, Response, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    api::{add::AddHandler, modify::ModifyHandler, retrieve::RetrieveHandler},
    error::{HttpError, PocketyError, PocketyError::Http},
};

static POCKET_BASE_URL: &str = "https://getpocket.com/v3/";

#[derive(Debug, Default)]
pub struct Auth {
    pub(crate) consumer_key:  String,
    pub(crate) request_token: Option<String>,
    pub(crate) access_token:  Option<String>,
}

#[derive(Debug)]
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

    // pub fn get_auth_url(&mut self) -> PocketResult<Url> {
    //     let request = try!(json::encode(&OAuthRequest {
    //         consumer_key: &*self.consumer_key,
    //         redirect_uri: "rustapi:finishauth",
    //         state: None
    //     }));

    //     self.request("https://getpocket.com/v3/oauth/request", &*request)
    //         .and_then(|r: PocketOAuthResponse| {
    //             let mut url = Url::parse("https://getpocket.com/auth/authorize").unwrap();
    //             url.set_query_from_pairs(
    //                 vec![
    //                     ("request_token", &*r.code),
    //                     ("redirect_uri", "rustapi:finishauth"),
    //                 ]
    //                 .into_iter(),
    //             );
    //             self.code = Some(r.code);
    //             Ok(url)
    //         })
    // }

    // pub fn authorize(&mut self) -> PocketResult<String> {
    //     let request = try!(json::encode(&PocketAuthorizeRequest {
    //         consumer_key: &*self.consumer_key,
    //         code: self.code.as_ref().map(|v| &**v).unwrap()
    //     }));

    //     match self.request("https://getpocket.com/v3/oauth/authorize", &*request) {
    //         Ok(r @ PocketAuthorizeResponse { .. }) => {
    //             self.access_token = Some(r.access_token);
    //             Ok(r.username)
    //         }
    //         Err(e) => Err(e),
    //     }
    // }
}
