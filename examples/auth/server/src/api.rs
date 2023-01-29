use async_session::{MemoryStore, Session, SessionStore};
use axum::{
    extract::{self, State},
    http::{header::SET_COOKIE, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use pockety::{Pockety, PocketyUrl};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::{error::Error, SessionData, COOKIE_NAME};

type Result<R> = std::result::Result<TypedResponse<R>, Error>;

#[derive(Debug, Clone)]
pub struct TypedResponse<B>
where
    B: Serialize,
{
    body: Option<B>,
    headers: Option<HeaderMap>,
    status_code: StatusCode,
}

impl<B> Default for TypedResponse<B>
where
    B: Serialize,
{
    fn default() -> Self {
        Self {
            body: None,
            headers: None,
            status_code: StatusCode::OK,
        }
    }
}

impl<B> IntoResponse for TypedResponse<B>
where
    B: Serialize,
{
    fn into_response(self) -> Response {
        let mut response = Json(self.body).into_response();
        if let Some(headers) = self.headers {
            *response.headers_mut() = headers;
        }
        *response.status_mut() = self.status_code;
        response
    }
}

pub async fn health_check() -> impl IntoResponse {
    "Healthy!"
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetRequestTokenResponse {
    request_token: String,
    auth_uri: String,
}

pub async fn get_request_token(
    State(pockety): State<Pockety>,
) -> Result<GetRequestTokenResponse> {
    let request_token = pockety.get_request_token(None).await?;

    let auth_uri = format!(
        "{}?request_token={request_token}&redirect_uri={}",
        PocketyUrl::AUTHORIZE,
        pockety.redirect_url
    );

    let response = GetRequestTokenResponse {
        request_token,
        auth_uri,
    };

    Ok(TypedResponse {
        body: Some(response),
        ..Default::default()
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenResponse {
    access_token: String,
    session_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccessTokenRequest {
    request_token: String,
}

pub async fn get_access_token(
    State(store): State<MemoryStore>,
    State(pockety): State<Pockety>,
    extract::Json(request): extract::Json<GetAccessTokenRequest>,
) -> Result<GetAccessTokenResponse> {
    let access_token = pockety
        .get_access_token(&request.request_token, None)
        .await?;

    let session_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let session_data = SessionData {
        session_id,
        access_token: access_token.clone(),
    };

    let mut session = Session::new();
    session.insert("session", &session_data)?;

    let cookie = store
        .store_session(session)
        .await
        .ok()
        .flatten()
        .ok_or(Error::Cookie("Failed to store session".to_string()))?;
    let cookie =
        format!("{COOKIE_NAME}={cookie}; SameSite=Lax; Path=/; HttpOnly");

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    let response = GetAccessTokenResponse {
        access_token,
        session_id: session_data.session_id,
    };

    Ok(TypedResponse {
        body: Some(response),
        ..Default::default()
    })
}
