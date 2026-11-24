pub mod exception;
pub mod extractors;
pub mod middlewares;

pub async fn graceful_shutdown() {
    if let Err(error) = wait_for_shutdown_signal().await {
        tracing::error!(%error, "Could not wait for a shutdown signal");
        std::future::pending::<()>().await;
    }

    tracing::info!("Shutdown requested; waiting for active requests");
}

async fn wait_for_shutdown_signal() -> app_error::R<()> {
    #[cfg(unix)]
    match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
        Ok(mut terminate) => {
            tokio::select! {
                result = tokio::signal::ctrl_c() => {
                    if let Err(error) = result {
                        tracing::error!(%error, "Could not wait for Ctrl-C");
                        terminate.recv().await;
                    }
                }
                _ = terminate.recv() => {},
            }
            return Ok(());
        }
        Err(error) => tracing::error!(%error, "Could not install SIGTERM handler"),
    }

    tokio::signal::ctrl_c().await?;
    Ok(())
}
