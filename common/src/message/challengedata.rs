use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug ,Clone)]
pub enum ChallengeData {
    SecretSumModulo(u128),
    SOS,
}
