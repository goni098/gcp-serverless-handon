use std::net::Ipv4Addr;

use axum::http::StatusCode;
use axum::routing::post;
use axum::{Router, middleware};
use database::repositories;
use http::exception::{HttpException, Rh};
use http::extractors::{Pool, PubsubEvent};
use http::middlewares::error_reporter;
use pvault::parsers::Event as PvaultEvent;
use shared::env::Env;
use shared::pubsub::PvaultEventPubsubData;

use crate::state::AppState;

mod state;

async fn handler(Pool(db): Pool, event: PubsubEvent) -> Rh<StatusCode> {
    let events =
        borsh::from_slice::<Vec<shared::pubsub::PvaultEventPubsubData>>(&event.message.data)
            .map_err(|_| HttpException::bad_request("invalid Pvault event batch"))?
            .into_iter()
            .filter_map(
                |PvaultEventPubsubData {
                     event_bytes,
                     sig,
                     ix,
                 }| {
                    PvaultEvent::parse(&event_bytes)
                    .inspect_err(
                        |error| tracing::debug!(%error, "Skipping unrecognized Pvault event data"),
                    )
                    .ok()
                    .map(|event| (sig,ix, event))
                },
            )
            .collect::<Vec<(String, i32, PvaultEvent)>>();

    for (sig, ix, event) in events {
        match event {
            PvaultEvent::Deposited(event) => {
                repositories::deposited_event::save(
                    &db,
                    &sig,
                    ix,
                    &event.mint.to_string(),
                    &event.depositor.to_string(),
                    event.amount,
                )
                .await?
            }
            PvaultEvent::InitializedVault(event) => {
                repositories::initialized_vault_event::save(
                    &db,
                    &sig,
                    ix,
                    &event.mint.to_string(),
                    &event.authority.to_string(),
                )
                .await?
            }
            PvaultEvent::Withdrawn(event) => {
                repositories::withdrawn_event::save(
                    &db,
                    &sig,
                    ix,
                    &event.mint.to_string(),
                    &event.authority.to_string(),
                    event.amount,
                )
                .await?
            }
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    shared::tracing::install();
    shared::env::load();

    let port = Env::Port.read_to::<u16>().expect("invalid port");
    let addr = (Ipv4Addr::UNSPECIFIED, port);

    let db_url = Env::DatabaseUrl.read().expect("DATABASE_URL is required");

    let state = AppState::new(&db_url)
        .await
        .expect("could not initialize app state");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let app = Router::new()
        .route("/", post(handler))
        .with_state(state)
        .layer(middleware::from_fn(error_reporter));

    tracing::info!(%port, "Listening on 0.0.0.0");

    axum::serve(listener, app)
        .with_graceful_shutdown(shared::functions::graceful_shutdown())
        .await?;

    tracing::info!("Server stopped");

    Ok(())
}
