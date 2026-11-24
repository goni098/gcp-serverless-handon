use std::convert::Infallible;

use axum::extract::{FromRef, FromRequestParts};
use sea_orm::DatabaseConnection;

pub struct Pool(pub DatabaseConnection);

impl<S> FromRequestParts<S> for Pool
where
    S: Send + Sync,
    DatabaseConnection: FromRef<S>,
{
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(DatabaseConnection::from_ref(state)))
    }
}
