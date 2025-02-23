mod decrypte;
mod player;
mod utils;
mod challenge;

use crate::player::handle_player;
use crate::utils::connect_to_server;
use common::message::actiondata::PlayerAction;
use common::message::MessageData;
use common::state::ClientState;
use common::utils::my_error::MyError;
use common::utils::utils::*;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use env_logger::Env;
use crate::decrypte::{exemple, DecodedView};

fn main() -> Result<(), MyError> {
    println!("Démarrage du client...");
    let addr = "localhost";
    let port = "8778";

    let mut stream = connect_to_server(addr, port)?;

    let mut state = ClientState::default();

    let team_name = "curious_broccoli".to_string();
    let message = build_message(MessageData::RegisterTeam { name: team_name })?;

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    send_message(&mut stream, &message)?;
    handle_response(&mut stream, &mut state)?;

    let (expected_players, token) = if let Some(team_info) = &state.team_info {
        (
            team_info.expected_players,
            team_info.registration_token.clone(),
        )
    } else {
        println!("Erreur : aucune information d'équipe disponible.");
        return Ok(());
    };

    let players = Arc::new(Mutex::new(Vec::new()));

    let radar_view = Arc::new(Mutex::new(DecodedView::default()));
    let (tx, rx) = channel();

    let player_threads: Vec<_> = (0..expected_players)
        .map(|i| {
            let players = Arc::clone(&players);
            let tx = tx.clone();
            let token = token.clone();
            let addr = addr.to_string();
            let port = port.to_string();
            let radar_view = Arc::clone(&radar_view);

            thread::spawn(move || {
                handle_player(i, token, &players, &addr, &port, tx,radar_view);
            })
        })
        .collect();

    let coordinator_thread = thread::spawn(move || {
        game_coordinator(rx, expected_players);
    });

    for handle in player_threads {
        handle.join().unwrap();
    }

    coordinator_thread.join().unwrap();
    Ok(())
}

fn game_coordinator(rx: Receiver<PlayerAction>, player_count: u32) {
    let mut active_players = player_count;

    while active_players > 0 {
        if let Ok(action) = rx.recv() {
            println!("Joueur {} action: {:?}", action.player_id, action.action);
        }
    }

    println!("Tous les joueurs ont terminé");
}
