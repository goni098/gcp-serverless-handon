use std::borrow::Cow;
use std::fmt;
use std::panic::Location;

pub type R<A> = Result<A, AppError>;

#[derive(thiserror::Error)]
pub struct AppError {
    #[source]
    context: AnyError,
    location: &'static Location<'static>,
}

impl AppError {
    #[track_caller]
    pub fn cust_err<E: ToString>(msg: E) -> Self {
        Self {
            context: AnyError::Cust(msg.to_string().into()),
            location: Location::caller(),
        }
    }

    #[track_caller]
    pub fn cust<E: Into<Cow<'static, str>>>(msg: E) -> Self {
        Self {
            context: AnyError::Cust(msg.into()),
            location: Location::caller(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.context.fmt(formatter)
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "Error: {:?}", self.context)?;
        writeln!(formatter, "Location: {}", self.location)?;

        Ok(())
    }
}

impl<E> From<E> for AppError
where
    E: Into<AnyError>,
{
    #[track_caller]
    fn from(value: E) -> Self {
        Self {
            context: value.into(),
            location: core::panic::Location::caller(),
        }
    }
}

#[allow(deprecated)]
#[derive(Debug, thiserror::Error)]
enum AnyError {
    #[error(transparent)]
    Database(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("{0}")]
    Cust(Cow<'static, str>),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),

    #[error(transparent)]
    Base64(#[from] base64::DecodeError),

    #[error(transparent)]
    TracingDirective(#[from] tracing_subscriber::filter::ParseError),

    #[error(transparent)]
    TracingInit(#[from] tracing_subscriber::util::TryInitError),

    #[error(transparent)]
    PubSubClientBuilder(#[from] google_cloud_gax::client_builder::Error),

    #[error(transparent)]
    PubSubPublish(#[from] google_cloud_pubsub::error::PublishError),

    #[error(transparent)]
    SolanaClient(#[from] solana_client::client_error::ClientError),

    #[error(transparent)]
    SolanaProgram(#[from] solana_sdk::program_error::ProgramError),

    #[error(transparent)]
    SolanaMessageCompile(#[from] solana_sdk::message::CompileError),

    #[error(transparent)]
    SolanaSigner(#[from] solana_sdk::signer::SignerError),

    #[error(transparent)]
    SolanaSignature(#[from] solana_signature::error::Error),
}
