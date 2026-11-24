use app_error::R;
use axum::extract::FromRef;
use database::DatabaseConnection;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub db: DatabaseConnection,
}

impl AppState {
    pub async fn new(db_url: &str) -> R<Self> {
        let db = database::connect_with_retry(db_url, None, None).await?;
        Ok(Self { db })
    }
}
