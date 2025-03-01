use crate::message::relativedirection::RelativeDirection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionData {
    MoveTo(RelativeDirection),
    SolveChallenge { answer: String },

}


pub struct PlayerAction {
    pub player_id: u32,
    pub action: ActionData,
}
