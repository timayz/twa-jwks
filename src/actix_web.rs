use futures_util::Future;
use serde::de::DeserializeOwned;
use std::pin::Pin;
use tracing::error;

use crate::Error;
use crate::JwksClient;

use actix_web::{
    error::ErrorBadRequest,
    http::{header, StatusCode},
    web::Data,
    Error as ActixError, FromRequest, HttpResponse, HttpResponseBuilder, ResponseError,
};

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        let mut res = HttpResponseBuilder::new(self.status_code());

        error!("{}", self);

        res.json(
            serde_json::json!({"code": self.status_code().as_u16(), "message": self.to_string()}),
        )
    }
}

pub struct JwtPayload<T: DeserializeOwned>(pub T);

impl<T: DeserializeOwned> FromRequest for JwtPayload<T> {
    type Error = ActixError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();
        let client = req
            .app_data::<Data<JwksClient>>()
            .expect("JwksClient not found in app data")
            .clone();

        Box::pin(async move {
            let token = match req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
            {
                Some(value) => value.replace("Bearer ", ""),
                _ => return Err(ErrorBadRequest("authorization is missing from header")),
            };

            let jwt = client.verify(&token).await.map_err(Error::from)?;
            let payload = jwt.payload().into::<T>().map_err(Error::from)?;

            Ok(Self(payload))
        })
    }
}
