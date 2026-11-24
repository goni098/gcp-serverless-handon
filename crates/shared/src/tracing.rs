use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, filter, fmt};

const CRATES: &[&str] = &[
    "api",
    "app_error",
    "database",
    "migration",
    "pvault",
    "pvault_emitter",
    "pvault_pusher",
    "pvault_subscriber",
    "shared",
    "shared_function",
];

pub fn install() {
    let out_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_filter(
            CRATES
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
