use std::io::Error;

use actix_web::{HttpResponse, HttpServer, App, web::Data, get, Responder};
use serde::Deserialize;
use twa_jwks::{JwksClient, actix_web::JwtPayload};

#[derive(Deserialize)]
struct JwtClaims {
    pub sub: String
}

#[get("/hello")]
async fn hello(JwtPayload(payload): JwtPayload<JwtClaims>) -> impl Responder {
    HttpResponse::Ok().body(format!("Hello, {}!", payload.sub))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let jwks_client = JwksClient::build(Some("http://127.0.0.1:6550/.well-known/jwks.json"))
        .await
        .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(jwks_client.clone()))
            .service(hello)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
