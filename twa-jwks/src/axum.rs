use axum::extract::FromRef;
use axum::http::{header, Request};
use serde::de::DeserializeOwned;
use serde_json::json;
use serde_json::Value;
use tracing::debug;

use crate::JwksClient;

use axum::{async_trait, extract::FromRequest, http::StatusCode};

pub struct JwtPayload<T: DeserializeOwned>(pub T);

#[async_trait]
impl<T, S, B> FromRequest<S, B> for JwtPayload<T>
where
    T: DeserializeOwned,
    B: Send + 'static,
    S: Send + Sync,
    JwksClient: FromRef<S>,
{
    type Rejection = (StatusCode, axum::Json<Value>);

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let token = match req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
        {
            Some(value) => value.replace("Bearer ", ""),
            _ => {
                let payload = json!({
                    "message": "Unauthorized",
                });

                return Err((StatusCode::UNAUTHORIZED, axum::Json(payload)));
            }
        };

        let client = JwksClient::from_ref(state);

        let jwt = match client.verify(&token).await {
            Ok(jwt) => jwt,
            Err(e) => {
                debug!("{}", e);

                let payload = json!({
                    "message": "Unauthorized",
                });

                return Err((StatusCode::UNAUTHORIZED, axum::Json(payload)));
            }
        };

        let payload = match jwt.payload().into::<T>() {
            Ok(payload) => Self(payload),
            Err(e) => {
                debug!("{}", e);

                let payload = json!({
                    "message": "Unauthorized",
                });

                return Err((StatusCode::UNAUTHORIZED, axum::Json(payload)));
            }
        };

        Ok(payload)
    }
}
