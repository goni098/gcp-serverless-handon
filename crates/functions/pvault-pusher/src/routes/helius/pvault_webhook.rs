use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use google_cloud_pubsub::client::Publisher;
use google_cloud_pubsub::model::Message;
use serde::Deserialize;
use serde_json::Value;
use shared::pubsub::PvaultEventPubsubData;
use shared_function::exception::Rh;
use shared_function::extractors::HeliusWebhookAuth;

#[derive(Deserialize, Debug)]
pub struct RawTransaction {
    meta: Option<TransactionMeta>,
    transaction: Transaction,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TransactionMeta {
    err: Value,
    log_messages: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct Transaction {
    signatures: Vec<String>,
}

impl RawTransaction {
    fn to_pubsub_data(&self) -> Vec<PvaultEventPubsubData> {
        let Some(TransactionMeta {
            err: Value::Null,
            log_messages: Some(logs),
        }) = &self.meta
        else {
            return vec![];
        };

        let Some(sig) = self.transaction.signatures.first() else {
            return vec![];
        };

        logs.iter()
            .enumerate()
            .filter_map(|(ix, log)| {
                let encoded = log.strip_prefix("Program data: ")?;
                let event_bytes = STANDARD.decode(encoded).ok()?;
                Some(PvaultEventPubsubData {
                    event_bytes,
                    ix: ix as i32,
                    sig: sig.to_string(),
                })
            })
            .collect()
    }
}

pub async fn handler(
    _: HeliusWebhookAuth,
    State(publisher): State<Publisher>,
    Json(transactions): Json<Vec<RawTransaction>>,
) -> Rh<StatusCode> {
    let data = transactions
        .iter()
        .flat_map(RawTransaction::to_pubsub_data)
        .collect::<Vec<PvaultEventPubsubData>>();

    if data.is_empty() {
        return Ok(StatusCode::OK);
    }

    let msg_data = bytes::Bytes::from_iter(borsh::to_vec(&data)?);

    let msg = Message::new().set_data(msg_data);

    publisher.publish(msg).await?;

    Ok(StatusCode::NO_CONTENT)
}
