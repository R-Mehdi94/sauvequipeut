use crate::message::actiondata::ActionData;
use crate::message::challengedata::ChallengeData;
use crate::message::hintdata::HintData;
 use crate::message::registerteam::{RegisterTeam, RegisterTeamResult};
use crate::message::subscribeplayer::{SubscribePlayer, SubscribePlayerResult};
use serde::{Deserialize, Serialize};

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
    RadarViewResult(String),
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ActionError {
    CannotPassThroughWall,
    CannotPassThroughOpponent,
    NoRunningChallenge,
    SolveChallengeFirst,
    InvalidChallengeSolution,
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
