mod decrypte;
mod player;
mod utils;
mod challenge;
mod hint;
mod position;
mod radar_view;
mod exploration_tracker;

use std::collections::{HashMap, HashSet};
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
use crate::challenge::TeamSecrets;
use crate::decrypte::{exemple, DecodedView, RadarCell};
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

     let (tx, rx) = channel();
    let team_secrets = Arc::new(TeamSecrets::new());
    let shared_compass = Arc::new(Mutex::new(None));
    let leader_id = Arc::new(Mutex::new(None));
    let shared_leader_action = Arc::new(Mutex::new(None));
    let shared_grid_size = Arc::new(Mutex::new(None));
    let position_tracker = Arc::new(Mutex::new(HashMap::new()));
    let visited_tracker = Arc::new(Mutex::new(HashMap::new()));
    let exit_position = Arc::new(Mutex::new(None));
     let labyrinth_map: Arc<Mutex<HashMap<(i32, i32), RadarCell>>> = Arc::new(Mutex::new(HashMap::new()));


    let player_threads: Vec<_> = (0..expected_players)
        .map(|i| {
            let players = Arc::clone(&players);
            let tx = tx.clone();
            let token = token.clone();
            let addr = addr.to_string();
            let port = port.to_string();
            let team_secrets_clone = Arc::clone(&team_secrets);
            let shared_compass_clone = Arc::clone(&shared_compass);
            let leader_id_clone =Arc::clone(&leader_id);
            let shared_leader_action_clone = Arc::clone(&shared_leader_action);
            let shared_grid_size_clone = Arc::clone(&shared_grid_size);
            let position_tracker_clone = Arc::clone(&position_tracker);
            let visited_tracker_clone = Arc::clone(&visited_tracker);
            let exit_position_clone=Arc::clone(&exit_position);
            let labyrinth_map_clone=Arc::clone(&labyrinth_map);
            thread::spawn(move || {
                handle_player(i, token, &players, &addr, &port, tx,team_secrets_clone , shared_compass_clone,leader_id_clone,shared_leader_action_clone,shared_grid_size_clone,
                              position_tracker_clone,visited_tracker_clone,exit_position_clone,labyrinth_map_clone);
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