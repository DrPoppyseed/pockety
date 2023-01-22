use std::{env, net::SocketAddr, time::Duration};

use axum_sessions::{async_session::MemoryStore, SessionLayer};
use pockety::{Pockety, PocketyUrl};

use axum::{
    extract::State,
    http::Method,
    response::IntoResponse,
    routing::{get, post},
    Router,
    Server,
};
use rand::{thread_rng, RngCore};
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer};

#[derive(Debug, Clone)]
struct AppState {
    pockety: Pockety,
}

#[tokio::main]
async fn main() {
    let store = MemoryStore::new();
    let mut secret = [0; 64];
    thread_rng()
        .try_fill_bytes(&mut secret)
        .expect("Failed to generate secret");
    let session_layer = SessionLayer::new(store, &secret);
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

    let app_state = AppState { pockety };

    let app = Router::new()
        .route("/", get(index))
        .route("/auth/pocket", post(get_request_token))
        .route("/auth/authorize", post(get_access_token))
        .layer(session_layer)
        .layer(cors_layer)
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
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

async fn get_request_token(
    State(mut app_state): State<AppState>,
) -> impl IntoResponse {
    let request_token = app_state
        .pockety
        .get_request_token(None)
        .await
        .expect("Failed to get request token");

    format!(
        "{}?request_token={request_token}&redirect_uri={}",
        PocketyUrl::AUTHORIZE,
        app_state.pockety.redirect_url
    )
}

async fn get_access_token(
    State(mut app_state): State<AppState>,
) -> impl IntoResponse {
    app_state
        .pockety
        .get_access_token(None)
        .await
        .expect("Failed to get access token");
}
