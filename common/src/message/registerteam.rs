use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeam {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeamResult {
    pub Ok: Option<RegisterTeamSuccess>,
    pub Err: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeamSuccess {
    pub expected_players: u32,
    pub registration_token: String,
}
