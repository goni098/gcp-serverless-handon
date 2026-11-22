use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct PvaultEventPubsubData {
    pub sig: String,
    pub ix: i32,
    pub event_bytes: Vec<u8>,
}
