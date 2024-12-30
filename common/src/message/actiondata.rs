use crate::message::relativedirection::RelativeDirection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ActionData {
    MoveTo(RelativeDirection),
    SolveChallenge { answer: String },
}
