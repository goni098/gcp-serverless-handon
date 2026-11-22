use axum::extract::FromRef;
use error::R;
use google_cloud_pubsub::client::Publisher;
use http::extractors::HeliusWebhookSecret;
use shared::env::Env;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub helius_webhook_secret: HeliusWebhookSecret,
    pub pvault_event_publisher: Publisher,
}

impl AppState {
    pub async fn new() -> R<Self> {
        let helius_webhook_secret = Env::HeliusWebhookSecret.read()?;
        let pvault_event_topic = Env::PVaultEventTopic.read()?;

        let helius_webhook_secret = HeliusWebhookSecret::new(helius_webhook_secret)?;
        let pvault_event_publisher = Publisher::builder(pvault_event_topic).build().await?;

        let state = AppState {
            helius_webhook_secret,
            pvault_event_publisher,
        };

        Ok(state)
    }
}
