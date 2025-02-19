mod decrypte;
mod player;
mod utils;

use crate::player::{handle_player, move_player};
use crate::utils::connect_to_server;
use common::message::MessageData;
use common::state::ClientState;
use common::utils::my_error::MyError;
use common::utils::utils::*;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() -> Result<(), MyError> {
    println!("Hello, world!");
    let addr = "localhost";
    let port = "8778";

    let mut stream = connect_to_server(addr, port)?;

    let mut state = ClientState::default();

    let team_name = "curious_broccoli".to_string();
    let message = build_message(MessageData::RegisterTeam { name: team_name })?;

    send_message(&mut stream, &message)?;
    handle_response(&mut stream, &mut state)?;

    let (expected_players, token) = if let Some(team_info) = &state.team_info {
        (
            team_info.expected_players.clone(),
            team_info.registration_token.clone(),
        )
    } else {
        println!("Erreur : aucune information d'équipe disponible.");
        return Ok(());
    };

    let subscribed_players = Arc::new(Mutex::new(Vec::new()));

    for i in 0..expected_players {
        let token = token.clone();
        let subscribed_players = Arc::clone(&subscribed_players); // Partage sécurisé entre threads des joueurs
        let addr = addr.to_string();
        let port = port.to_string();

        let play = thread::spawn(move || {
            handle_player(i, token, &subscribed_players, &addr, &port);
            for player in subscribed_players.lock().unwrap().iter_mut() {
                move_player(player)
            }
        });
        play.join().unwrap();
    }

    loop {
        thread::sleep(std::time::Duration::from_secs(1));
    }
}
