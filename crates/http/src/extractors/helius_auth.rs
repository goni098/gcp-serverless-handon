use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{HeaderValue, header};
use error::{AppError, R};

use crate::exception::HttpException;

#[derive(Clone)]
pub struct HeliusWebhookSecret(HeaderValue);

impl HeliusWebhookSecret {
    pub fn new(secret: String) -> R<Self> {
        if secret.is_empty() || secret.trim() != secret {
            return Err(AppError::cust("invalid secret"));
        }

        let mut secret = HeaderValue::try_from(secret).map_err(AppError::cust_err)?;

        secret.set_sensitive(true);

        Ok(Self(secret))
    }
}

pub struct HeliusWebhookAuth;

impl<S: Send + Sync> FromRequestParts<S> for HeliusWebhookAuth
where
    HeliusWebhookSecret: FromRef<S>,
{
    type Rejection = HttpException;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let HeliusWebhookSecret(expected_secret) = HeliusWebhookSecret::from_ref(state);

        let secret = parts
            .headers
            .get(header::AUTHORIZATION)
            .ok_or_else(|| HttpException::unauthorized("missing authorization"))?;

        if !bool::from(expected_secret.as_bytes().eq(secret.as_bytes())) {
            return Err(HttpException::forbidden(
                "invalid Helius webhook authorization",
            ));
        }

        Ok(Self)
    }
}
