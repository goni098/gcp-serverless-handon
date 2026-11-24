use axum::extract::FromRequest;
use cloudevents::{AttributesReader, Data};
use google_cloud_pubsub::model::Message;
use serde::{Deserialize, Serialize};

use crate::exception::HttpException;

pub const EVENT_TYPE: &str = "google.cloud.pubsub.topic.v1.messagePublished";

#[derive(Deserialize, Serialize, Debug)]
pub struct PubsubEvent {
    #[serde(deserialize_with = "deserialize_pubsub_message")]
    pub message: Message,
    pub subscription: String,
}

fn deserialize_pubsub_message<'de, D>(deserializer: D) -> Result<Message, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut message = serde_json::Map::deserialize(deserializer)?;

    // Wrapped push requests include both spellings, which the SDK treats as duplicate fields.
    for (canonical, alias) in [("messageId", "message_id"), ("publishTime", "publish_time")] {
        if message
            .get(canonical)
            .is_some_and(|value| message.get(alias) == Some(value))
        {
            message.remove(alias);
        }
    }

    serde_json::from_value(serde_json::Value::Object(message)).map_err(serde::de::Error::custom)
}

impl<S> FromRequest<S> for PubsubEvent
where
    S: Send + Sync,
{
    type Rejection = HttpException;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        // Authenticated Pub/Sub push subscriptions send a wrapped JSON envelope.
        // Eventarc sends the same payload inside a CloudEvent.
        let is_cloud_event = req.headers().contains_key("ce-type")
            || req
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.starts_with("application/cloudevents+json"));

        if !is_cloud_event {
            return axum::Json::<Self>::from_request(req, state)
                .await
                .map(|axum::Json(event)| event)
                .map_err(|_| HttpException::bad_request("invalid pub/sub push envelope"));
        }

        let event = cloudevents::Event::from_request(req, state)
            .await
            .map_err(|_| HttpException::bad_request("invalid cloud event"))?;

        if event.ty() != EVENT_TYPE {
            return Err(HttpException::bad_request("invalid pub/sub event type"));
        }

        let data = event
            .data()
            .ok_or_else(|| HttpException::bad_request("missing cloud event data"))?;

        let data = match data {
            Data::Json(value) => serde_json::from_value(value.clone())?,
            Data::Binary(bytes) => serde_json::from_slice(bytes)?,
            Data::String(text) => serde_json::from_str(text)?,
        };

        Ok(data)
    }
}
