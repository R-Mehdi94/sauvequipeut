use crate::message::actiondata::ActionData;
use crate::message::challengedata::ChallengeData;
use crate::message::hintdata::HintData;
use crate::message::radar_view::RadarViewResult;
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

#[derive(Serialize, Deserialize)]
pub enum Message {
    RegisterTeam(RegisterTeam),
    RegisterTeamResult(RegisterTeamResult),
    SubscribePlayerResult(SubscribePlayerResult),
    SubscribePlayer(SubscribePlayer),
    Hint(HintData),
    Action(ActionData),
    Challenge(ChallengeData),
    RadarViewResult(String),
}
