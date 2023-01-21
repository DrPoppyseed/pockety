use std::{env, net::SocketAddr};

use pockety::Pockety;

use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
    Server,
};

#[derive(Debug, Clone)]
struct AppState {
    pockety_client: Pockety,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/auth/pocket", get(auth_pocket))
        .route("/auth/authorized", get(auth_authorized))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
