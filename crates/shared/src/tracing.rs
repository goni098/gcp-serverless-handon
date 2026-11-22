use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, filter, fmt};

pub fn install() {
    let crates = [
        "api",
        "cron",
        "database",
        "error",
        "http",
        "migration",
        "pubsub",
        "shared",
    ];

    let out_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(
            crates
                .iter()
                .fold(EnvFilter::from_default_env(), |filter, c| {
                    filter.add_directive(c.to_string().parse().expect("Invalid filter directive"))
                }),
        )
        .with_filter(filter::filter_fn(|metadata| {
            *metadata.level() != Level::ERROR
        }));

    let err_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(filter::LevelFilter::ERROR);

    tracing_subscriber::registry()
        .with(out_layer)
        .with(err_layer)
        .init();
}
