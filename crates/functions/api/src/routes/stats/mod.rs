use axum::Router;
use axum::routing::post;

use crate::state::AppState;

mod event_counts;

pub fn routes() -> Router<AppState> {
    Router::new().route("/stats/event_counts", post(event_counts::handler))
}
