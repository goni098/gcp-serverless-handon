use axum::Router;
use axum::routing::post;

use crate::state::AppState;

mod pvault_webhook;

pub fn routes() -> Router<AppState> {
    Router::new().route("/helius/pvault", post(pvault_webhook::handler))
}
