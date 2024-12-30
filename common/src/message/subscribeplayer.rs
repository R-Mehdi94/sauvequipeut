use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SubscribePlayer {
    pub name: String,
    pub registration_token: String,
}

#[derive(Serialize, Deserialize)]
pub enum SubscribePlayerResult {
    Ok,
    Err(String),
}
