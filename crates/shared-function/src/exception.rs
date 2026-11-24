use std::borrow::Cow;
use std::sync::Arc;

use app_error::AppError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpException {
    #[error(transparent)]
    App(AppError),

    #[error("BadRequest: {0}")]
    BadRequest(Cow<'static, str>),

    #[error("Unauthorized: {0}")]
    Unauthorized(Cow<'static, str>),

    #[error("Forbidden: {0}")]
    Forbidden(Cow<'static, str>),

    #[error("{0}")]
    Internal(Cow<'static, str>),
}

pub type Rh<A> = Result<A, HttpException>;

impl IntoResponse for HttpException {
    fn into_response(self) -> axum::response::Response {
        let (code, msg, can_attach_ext) = match &self {
            HttpException::App(_error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error".into(),
                true,
            ),
            HttpException::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone(), true),
            HttpException::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone(), false),
            HttpException::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone(), false),
            HttpException::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone(), false),
        };

        let mut response = (code, msg).into_response();

        if can_attach_ext {
            response.extensions_mut().insert(Arc::new(self));
        }

        response
    }
}

impl HttpException {
    pub fn is_server_error(&self) -> bool {
        matches!(self, HttpException::App(_) | HttpException::Internal(_))
    }

    pub fn bad_request<E: Into<Cow<'static, str>>>(msg: E) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn unauthorized<E: Into<Cow<'static, str>>>(msg: E) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn forbidden<E: Into<Cow<'static, str>>>(msg: E) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn internal<E: Into<Cow<'static, str>>>(msg: E) -> Self {
        Self::Internal(msg.into())
    }
}

impl<E> From<E> for HttpException
where
    E: Into<AppError>,
{
    fn from(value: E) -> Self {
        Self::App(value.into())
    }
}
