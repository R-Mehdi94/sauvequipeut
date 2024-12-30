use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ChallengeData {
    SecretSumModulo(u64),
    SOS,
}
