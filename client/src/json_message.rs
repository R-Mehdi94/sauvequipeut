use serde::{Deserialize, Serialize};
/*use std::fmt::Formatter;

impl std::fmt::Display for RegisterTeam {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RegisterTeam({})", self.name,)
    }
}*/

#[derive(Serialize, Deserialize)]
pub struct RegisterTeam {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterTeamResult {
    pub Ok: Option<RegisterTeamSuccess>,
    pub Err: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterTeamSuccess {
    pub expected_players: u32,
    pub registration_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubscribePlayer {
    pub name: String,
    pub registration_token: String,
}

#[derive(Serialize, Deserialize)]
pub struct SubscribePlayerResult {}

#[derive(Serialize, Deserialize)]
pub enum Message {
    RegisterTeam(RegisterTeam),
    RegisterTeamResult(RegisterTeamResult),
    SubscribePlayer(SubscribePlayer),
}
