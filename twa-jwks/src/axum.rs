use std::convert::Infallible;

use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;
use axum::{Extension, RequestPartsExt};
use serde::de::DeserializeOwned;
use tracing::debug;

use crate::JwksClient;

use axum::{async_trait, http::StatusCode};

pub struct JwtPayloadOption<T: DeserializeOwned>(pub Option<T>);

#[async_trait]
impl<S, T> FromRequestParts<S> for JwtPayloadOption<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let client = parts
            .extract::<Extension<JwksClient>>()
            .await
            .expect("JwksClient not configured correctly");

        let Some(authorization) = parts.headers.get(header::AUTHORIZATION) else {
            return Ok(Self(None));
        };

        let Ok(authorization) = authorization.to_str() else {
            return Ok(Self(None));
        };

        let token = authorization.replace("Bearer ", "");

        let jwt = match client.verify(&token).await {
            Ok(jwt) => jwt,
            Err(e) => {
                debug!("{}", e);

                return Ok(Self(None));
            }
        };

        match jwt.payload().into::<T>() {
            Ok(payload) => Ok(Self(Some(payload))),
            Err(e) => {
                debug!("{}", e);

                return Ok(Self(None));
            }
        }
    }
}

pub struct JwtPayload<T: DeserializeOwned>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for JwtPayload<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = (StatusCode, axum::response::Html<&'static str>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Ok(payload) = JwtPayloadOption::<T>::from_request_parts(parts, state).await else {
            return Err((
                StatusCode::UNAUTHORIZED,
                axum::response::Html("Unauthorized"),
            ));
        };

        let Some(payload) = payload.0 else {
            return Err((
                StatusCode::UNAUTHORIZED,
                axum::response::Html("Unauthorized"),
            ));
        };

        Ok(Self(payload))
    }
}
