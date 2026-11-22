use std::sync::Arc;

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::exception::HttpException;

pub async fn error_reporter(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let endpoint = request.uri().path().to_owned();
    let mut response = next.run(request).await;

    if let Some(error) = response.extensions_mut().remove::<Arc<HttpException>>() {
        tracing::error!(
            "Request handler failed method={}, endpoint={}, status={}\n error= {:#?}",
            method,
            endpoint,
            response.status().as_u16(),
            error
        )
    }

    response
}
