use std::net::Ipv4Addr;

use app_error::R;
use axum::routing::get;
use axum::{Router, middleware};
use shared::env::Env;

use crate::state::AppState;

mod routes;
mod state;

#[tokio::main]
async fn main() -> R<()> {
    shared::tracing::install()?;
    shared::env::load();

    let port = Env::Port.read_to::<u16>()?;

    let state = AppState::new().await?;

    let listener = tokio::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;

    let app = Router::new()
        .route("/", get(async || "Pvault event pusher is running"))
        .merge(routes::helius::routes())
        .layer(middleware::from_fn(
            shared_function::middlewares::error_reporter,
        ))
        .with_state(state);

    tracing::info!(%port, "Listening on 0.0.0.0");
    axum::serve(listener, app)
        .with_graceful_shutdown(shared_function::graceful_shutdown())
        .await?;
    Ok(())
}
