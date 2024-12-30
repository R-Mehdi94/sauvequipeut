use serde::{Deserialize, Serialize};

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
