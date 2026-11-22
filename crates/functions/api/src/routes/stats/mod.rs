use axum::Router;
use axum::routing::get;

use crate::state::AppState;

mod event_counts;

pub fn routes() -> Router<AppState> {
    Router::new().route("/stats/event_counts", get(event_counts::handler))
}
