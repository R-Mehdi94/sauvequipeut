use crate::message::{RegisterTeamSuccess, SubscribePlayer};

#[derive(Default)]
pub struct ClientState {
    pub team_info: Option<RegisterTeamSuccess>,
    pub subscribed_players: Vec<SubscribePlayer>,
}
