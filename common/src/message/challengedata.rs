use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ChallengeData {
    SecretSumModulo(u64),
    SOS,
}
