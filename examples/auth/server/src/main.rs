use std::{env, net::SocketAddr};

use async_session::{async_trait, MemoryStore, Session, SessionStore};
use pockety::{Pockety, PocketyUrl};

use axum::{
    extract::{
        rejection::TypedHeaderRejectionReason,
        FromRef,
        FromRequestParts,
        State, FromRequest,
    },
    headers,
    http::{
        header::{self, SET_COOKIE},
        request::Parts,
        HeaderMap,
        Method,
    },
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json,
    RequestPartsExt,
    Router,
    Server,
    TypedHeader,
};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;

static COOKIE_NAME: &str = "SESSION";

#[derive(Debug, Clone)]
struct AppState {
    pockety: Pockety,
    store: MemoryStore,
}

#[tokio::main]
async fn main() {
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
        .allow_methods([Method::GET, Method::POST]);

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
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> impl IntoResponse {
    "Hello, World!"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    session_id: String,
    request_token: String,
    access_token: Option<String>,
}

#[derive(Serialize)]
pub struct GetRequestTokenResponse {
    request_token: String,
    auth_uri: String,
    session_id: String,
}

async fn get_request_token(
    State(store): State<MemoryStore>,
    State(mut pockety): State<Pockety>,
) -> impl IntoResponse {
    let request_token = pockety
        .get_request_token(None)
        .await
        .expect("Failed to get request token");

    let mut session_id = [0; 16];
    thread_rng()
        .try_fill_bytes(&mut session_id)
        .expect("Failed to generate session id");
    let session_id: String = String::from_utf8(session_id.to_vec()).unwrap();

    let auth_uri = format!(
        "{}?request_token={request_token}&redirect_uri={}",
        PocketyUrl::AUTHORIZE,
        pockety.redirect_url
    );

    let response = GetRequestTokenResponse {
        request_token: request_token.clone(),
        auth_uri,
        session_id: session_id.clone(),
    };

    let session_data = SessionData {
        session_id,
        request_token,
        access_token: None,
    };

    let mut session = Session::new();
    session.insert("session", &session_data).unwrap();

    let cookie = store.store_session(session).await.unwrap().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, Json(response)).into_response()
}

async fn get_access_token(
    State(store): State<MemoryStore>,
    State(mut pockety): State<Pockety>,
) -> impl IntoResponse {
    pockety
        .get_access_token(None)
        .await
        .expect("Failed to get access token");
}

struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary("/auth/pocket").into_response()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for SessionData
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthRedirect;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => AuthRedirect,
                    _ => {
                        panic!("unexpected error getting Cookie header(s): {e}")
                    }
                },
                _ => panic!("unexpected error getting cookies: {e}"),
            })?;
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(AuthRedirect)?;

        let session = store
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(AuthRedirect)?;

        let session =
            session.get::<SessionData>("session").ok_or(AuthRedirect)?;

        Ok(session)
    }
}

#[async_trait]
impl<P> FromRe