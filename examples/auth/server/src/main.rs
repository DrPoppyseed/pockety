use std::{env, net::SocketAddr};

use async_session::{async_trait, MemoryStore, Session, SessionStore};
use error::Error;
use pockety::{Pockety, PocketyUrl};

use axum::{
    extract::{
        self,
        rejection::TypedHeaderRejectionReason,
        FromRef,
        FromRequestParts,
        State,
    },
    headers,
    http::{
        header::{COOKIE, SET_COOKIE},
        request::Parts,
        HeaderMap,
        Method,
        StatusCode,
    },
    response::{IntoResponse, Response},
    routing::{get, post},
    Json,
    RequestPartsExt,
    Router,
    Server,
    TypedHeader,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng, RngCore};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace};
use tracing::Level;

mod error;

static COOKIE_NAME: &str = "POCKETY_AUTH";

type Result<R> = std::result::Result<TypedResponse<R>, Error>;

#[derive(Debug, Clone)]
struct AppState {
    pockety: Pockety,
    store: MemoryStore,
}

impl FromRef<AppState> for Pockety {
    fn from_ref(state: &AppState) -> Self {
        state.pockety.clone()
    }
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

#[derive(Debug, Clone)]
struct TypedResponse<B>
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let store = MemoryStore::new();
    let mut secret = [0; 64];
    thread_rng()
        .try_fill_bytes(&mut secret)
        .expect("Failed to generate secret");
    let cors_layer = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "https://getpocket.com".parse().unwrap(),
        ])
        .allow_headers(["content-type".parse().unwrap()])
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true);

    let pockety = Pockety::new(
        &env::var("POCKET_CONSUMER_KEY").expect("Missing POCKET_CONSUMER_KEY"),
        &env::var("POCKET_REDIRECT_URI").expect("Missing POCKET_REDIRECT_URI"),
    );

    let app_state = AppState { pockety, store };

    let app = Router::new()
        .route("/", get(index))
        .route("/auth/pocket", post(get_request_token))
        .route("/auth/authorize", post(get_access_token))
        .layer(cors_layer)
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(
                    trace::DefaultMakeSpan::new().level(Level::INFO),
                )
                .on_response(
                    trace::DefaultOnResponse::new().level(Level::INFO),
                ),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Listening on {addr}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to launch server");
}

async fn index() -> impl IntoResponse {
    "Hello, World!"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    session_id: String,
    access_token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetRequestTokenResponse {
    request_token: String,
    auth_uri: String,
}

async fn get_request_token(
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

async fn get_access_token(
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

#[async_trait]
impl<S> FromRequestParts<S> for SessionData
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => {
                        Error::Cookie("missing Cookie header".to_string())
                    }
                    _ => Error::Cookie(
                        "unexpected error getting Cookie header(s): {e}"
                            .to_string(),
                    ),
                },
                _ => Error::Cookie(
                    "unexpected error getting cookies: {e}".to_string(),
                ),
            })?;

        let session_cookie = cookies
            .get(COOKIE_NAME)
            .ok_or(Error::Cookie("missing cookie".to_string()))?;

        let session = store
            .load_session(session_cookie.to_string())
            .await
            .ok()
            .flatten()
            .ok_or(Error::Cookie("failed to load session".to_string()))?;

        session
            .get::<SessionData>("session")
            .ok_or(Error::Cookie("session not found".to_string()))
    }
}
