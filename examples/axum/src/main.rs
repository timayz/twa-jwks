use axum::{response::IntoResponse, routing::get, Json, Router};
use serde::Deserialize;
use twa_jwks::{axum::JwtPayload, JwksClient};

#[derive(Deserialize)]
struct JwtClaims {
    pub sub: String,
}

async fn hello(JwtPayload(payload): JwtPayload<JwtClaims>) -> impl IntoResponse {
    Json(format!("hello from axum, {}", payload.sub))
}

#[tokio::main]
async fn main() {
    let jwks_client = JwksClient::new("http://127.0.0.1:6550/.well-known/jwks.json")
        .await
        .unwrap();

    let app = Router::new()
        .route("/hello", get(hello))
        .with_state(jwks_client);

    axum::Server::bind(&([127, 0, 0, 1], 3001).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
