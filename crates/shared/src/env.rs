use std::borrow::Cow;
use std::str::FromStr;

use error::{AppError, R};

#[derive(Clone, Copy, Debug)]
pub enum Env {
    Port,
    DatabaseUrl,
    JwtSecret,
    SolRpcUrl,
    SolPk,
    HeliusWebhookSecret,
    PVaultEventTopic,
}

pub fn load() {
    match dotenvy::dotenv() {
        Ok(path) => tracing::info!("Loaded .env file from: {}", path.display()),
        Err(_) => tracing::info!("No .env file found, using system environment variables"),
    }
}

pub fn is_local() -> bool {
    std::env::var("LOCAL").is_ok_and(|flag| flag == "1")
}

impl Env {
    fn key(&self) -> Cow<'static, str> {
        match self {
            Self::DatabaseUrl => "DATABASE_URL".into(),
            Self::JwtSecret => "JWT_SECRET".into(),
            Self::Port => "PORT".into(),
            Self::HeliusWebhookSecret => "HELIUS_WEBHOOK_SECRET".into(),
            Self::PVaultEventTopic => "PVAULT_EVENT_TOPIC".into(),
            Self::SolRpcUrl => "SOL_RPC_URL".into(),
            Self::SolPk => "SOL_PK".into(),
        }
    }

    pub fn read(&self) -> R<String> {
        read(self.key())
    }

    pub fn read_to<T>(&self) -> R<T>
    where
        T: FromStr,
    {
        read_to(self.key())
    }
}

fn read(var: impl AsRef<str>) -> R<String> {
    std::env::var(var.as_ref()).map_err(|_| AppError::cust(format!("missing {} env", var.as_ref())))
}

fn read_to<T>(var: impl AsRef<str>) -> R<T>
where
    T: FromStr,
{
    read(&var).and_then(|v| {
        v.parse()
            .map_err(|_| AppError::cust(format!("can not convert env var {}", var.as_ref())))
    })
}
