use crate::utils::connect_to_server;
use common::message::MessageData;
use common::state::ClientState;
use common::utils::my_error::MyError;
use common::utils::utils::{build_message, handle_response, send_message};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub struct Player {
    pub name: String,
    pub registration_token: String,
    pub stream: TcpStream,
}

pub fn handle_player(
    player_id: u32,
    token: String,
    subscribed_players: Arc<Mutex<Vec<Player>>>,
    addr: &str,
    port: &str,
) -> Result<(), MyError> {
    let stream = connect_to_server(addr, port)?;
    println!("Joueur {} connecté.", player_id);

    let player_name = format!("Player_{}", player_id);
    let subscribe_message = build_message(MessageData::SubscribePlayer {
        name: player_name.clone(),
        registration_token: token.clone(),
    })?;

    let mut player_stream = stream.try_clone()?;
    send_message(&mut player_stream, &subscribe_message)?;

    // Ajouter le joueur à la liste partagée
    {
        let mut players = subscribed_players.lock().unwrap();
        players.push(Player {
            name: player_name.clone(),
            registration_token: token.clone(),
            stream: player_stream.try_clone()?,
        });
    }

    // Boucle pour gérer les messages du serveur
    loop {
        handle_response(&mut player_stream, &mut ClientState::default())?;
    }
}
