use crate::message::actiondata::ActionData;
use crate::message::challengedata::ChallengeData;
use crate::message::hintdata::HintData;
 use crate::message::registerteam::{RegisterTeam, RegisterTeamResult};
use crate::message::subscribeplayer::{SubscribePlayer, SubscribePlayerResult};
use serde::{Deserialize, Serialize};
use crate::message::actionerror::ActionError;

pub enum MessageData {
    RegisterTeam {
        name: String,
    },
    SubscribePlayer {
        name: String,
        registration_token: String,
    },
    Hint(HintData),
    Action(ActionData),
    Challenge(ChallengeData),
}


#[derive(Serialize, Deserialize, Debug)]

pub enum Message {
    RegisterTeam(RegisterTeam),
    RegisterTeamResult(RegisterTeamResult),
    SubscribePlayerResult(SubscribePlayerResult),
    SubscribePlayer(SubscribePlayer),
    Hint(HintData),
    Action(ActionData),
    Challenge(ChallengeData),
    RadarViewResult(String),
    ActionError(ActionError)
}
