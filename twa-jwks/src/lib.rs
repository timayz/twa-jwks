#![forbid(unsafe_code)]

use jwt::Jwt;
use keyset::KeyStore;
use std::sync::Arc;
use thiserror::Error as ThisError;
use tokio::sync::RwLock;
use tracing::error;

pub mod error;
pub mod jwt;
pub mod keyset;

#[cfg(feature = "actix-web")]
pub mod actix_web;

#[cfg(feature = "axum")]
pub mod axum;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("reqwest: {0}")]
    Reqwest(reqwest::Error),

    #[error("jwks_client: {0}")]
    Jwks(error::Error),

    #[error("{0}")]
    Unknown(String),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}

impl From<error::Error> for Error {
    fn from(e: error::Error) -> Self {
        Error::Jwks(e)
    }
}

///
/// ```rust
///
/// use twa_jwks::JwksClient;
///
/// let jwks_client = JwksClient::new("http://127.0.0.1:4456/.well-known/jwks.json").await.unwrap();
/// ```
#[derive(Clone)]
pub struct JwksClient {
    inner: Arc<RwLock<KeyStore>>,
    insecure: bool,
}

impl JwksClient {
    pub async fn new<U: Into<String>>(url: U) -> Result<Self, Error> {
        Self::build(Some(url)).await
    }

    pub async fn insecure() -> Result<Self, Error> {
        Self::build(None::<String>).await
    }

    pub async fn build<U: Into<String>>(url: Option<U>) -> Result<Self, Error> {
        match url {
            Some(url) => {
                let store = KeyStore::new_from(url.into()).await?;

                Ok(Self {
                    inner: Arc::new(RwLock::new(store)),
                    insecure: false,
                })
            }
            _ => {
                let store = KeyStore::new();

                Ok(Self {
                    inner: Arc::new(RwLock::new(store)),
                    insecure: true,
                })
            }
        }
    }

    pub async fn verify(&self, token: &str) -> Result<Jwt, error::Error> {
        let read = self.inner.read().await;

        if self.insecure {
            return read.decode(token);
        }

        if read.should_refresh().unwrap_or(false) {
            drop(read);

            let mut guard = self.inner.write().await;
            guard.load_keys().await?;

            drop(guard);
        }

        let read = self.inner.read().await;

        read.verify(token)
    }
}
