use std::{env, net::SocketAddr};

use async_session::{async_trait, MemoryStore, SessionStore};
use error::Error;
use pockety::Pockety;

use axum::{
    extract::{rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts},
    headers,
    http::{header::COOKIE, request::Parts, Method},
    routing::{get, post},
    RequestPartsExt, Router, Server, TypedHeader,
};
use rand::{thread_rng, RngCore};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace};
use tracing::Level;

use crate::api::{get_access_token, get_articles, get_request_token, health_check};

mod api;
mod error;

pub static COOKIE_NAME: &str = "POCKETY_AUTH";

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
        env::var("POCKET_CONSUMER_KEY").expect("Missing POCKET_CONSUMER_KEY"),
        env::var("POCKET_REDIRECT_URI").expect("Missing POCKET_REDIRECT_URI"),
    )
    .expect("failed to create Pockety instance.");

    let app_state = AppState { pockety, store };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/articles", get(get_articles))
        .route("/auth/pocket", post(get_request_token))
        .route("/auth/authorize", post(get_access_token))
        .layer(cors_layer)
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Listening on {addr}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to launch server");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    session_id: String,
    access_token: String,
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
    ) -> Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);

        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => {
                        Error::Cookie("missing Cookie header".to_string())
                    }
                    _ => {
                        Error::Cookie("unexpected error getting Cookie header(s): {e}".to_string())
                    }
                },
                _ => Error::Cookie("unexpected error getting cookies: {e}".to_string()),
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
