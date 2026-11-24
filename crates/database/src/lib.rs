use std::time::Duration;

use app_error::{AppError, R};
use sea_orm::Database;
pub use sea_orm::DatabaseConnection;

pub mod entity;
pub mod repositories;

#[derive(Debug, Clone, Default)]
pub struct ConnectOptions {
    pub max_pool_size: Option<u32>,
    pub pool_wait_timeout: Option<Duration>,
}

/// Startup retry settings. Keep `total_timeout` within the service's startup budget.
#[derive(Debug, Clone)]
pub struct RetryOptions {
    pub total_timeout: Duration,
    pub attempt_timeout: Duration,
    pub retry_delay: Duration,
}

impl Default for RetryOptions {
    fn default() -> Self {
        Self {
            total_timeout: Duration::from_secs(180),
            attempt_timeout: Duration::from_secs(10),
            retry_delay: Duration::from_secs(5),
        }
    }
}

/// Connect with bounded retries, including when an individual attempt stalls.
/// Useful during Cloud Run startup while Direct VPC egress becomes available.
/// `None` uses SeaORM's pool defaults and the default retry settings.
pub async fn connect_with_retry(
    database_url: &str,
    options: Option<ConnectOptions>,
    retry_options: Option<RetryOptions>,
) -> R<DatabaseConnection> {
    let retry = retry_options.unwrap_or_default();
    let mut last_failure = String::from("no connection attempt completed");

    tokio::time::timeout(retry.total_timeout, async {
        loop {
            match tokio::time::timeout(
                retry.attempt_timeout,
                connect(database_url, options.clone()),
            )
            .await
            {
                Ok(Ok(db)) => return db,
                Ok(Err(err)) => last_failure = err.to_string(),
                Err(_) => {
                    last_failure = format!(
                        "connection attempt timed out after {:?}",
                        retry.attempt_timeout
                    );
                }
            }
            tracing::warn!(
                retry_delay = ?retry.retry_delay,
                "Database unavailable during startup; retrying"
            );
            tokio::time::sleep(retry.retry_delay).await;
        }
    })
    .await
    .map_err(|_| {
        AppError::cust(format!(
            "database startup timed out after {:?}: {last_failure}",
            retry.total_timeout
        ))
    })
}

async fn connect(database_url: &str, options: Option<ConnectOptions>) -> R<DatabaseConnection> {
    let mut config = sea_orm::ConnectOptions::new(database_url);

    if let Some(options) = options {
        if let Some(max_pool_size) = options.max_pool_size {
            config.max_connections(max_pool_size);
        }
        if let Some(timeout) = options.pool_wait_timeout {
            config.acquire_timeout(timeout);
        }
    }

    let db = Database::connect(config).await?;

    Ok(db)
}
