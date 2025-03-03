mod player;
mod utils;

use std::collections::HashMap;
use crate::player::handle_player;
use crate::utils::connect_to_server;
use common::message::actiondata::PlayerAction;
use common::message::MessageData;
use common::state::ClientState;
use common::utils::my_error::MyError;
use common::utils::utils::*;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::{env, thread};
use std::thread::sleep;
use std::time::Duration;
use env_logger::Env;
use algorithms::decrypte::RadarCell;
use algorithms::challenge::TeamSecrets;
use algorithms::exploration_tracker::ExplorationTracker;

/// Point d'entrée principal du programme.
///
/// Cette fonction :
/// - **Se connecte au serveur** et inscrit l'équipe.
/// - **Attend les joueurs** et démarre le jeu.
/// - **Lance un thread pour chaque joueur**.
/// - **Gère la coordination du jeu**.
///
/// # Retourne
/// - `Ok(())` en cas de succès.
/// - `Err(MyError)` en cas d'erreur de connexion.
///
/// # Exemple
/// ```no_run
/// fn main() -> Result<(), MyError> {
///     main()
/// }
/// ```
fn main() -> Result<(), MyError> {
    println!("Démarrage du client...");


    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: client [server_address] [port (DEFAULT 8778)]");
        return Ok(());
    }

    let addr = &args[1];
    let mut port = "8778";
    if args.len() == 3 {
        port = &args[2];
    }

    //let addr = "localhost";
    //let port = "8778";


    let mut stream = connect_to_server(addr, port)?;

    let mut state = ClientState::default();

    let mut line = String::new();
    println!("Enter your team name :");
    std::io::stdin().read_line(&mut line)?;
    let team_name = line.trim().to_string();

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

    println!("Nombre de joueurs : {}",expected_players);

    sleep(Duration::from_secs(2));
    println!("{}", "⚠️ Sortez vite du labyrinthe avant que...");
    sleep(Duration::from_secs(5));

    println!("La partie commence dans :");

    for i in (0..3).rev() {
        println!("{}", i+1 );
        sleep(Duration::from_secs(1));
    }

    println!("{}", "🏁 GO GO GO !");
    sleep(Duration::from_secs(2));

    // 📌 Initialisation des variables partagées
    let players = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = channel();
    let team_secrets = Arc::new(TeamSecrets::new());
    let shared_compass = Arc::new(Mutex::new(None));
    let leader_id = Arc::new(Mutex::new(None));
    let shared_leader_action = Arc::new(Mutex::new(None));
    let shared_grid_size = Arc::new(Mutex::new(None));
    let position_tracker = Arc::new(Mutex::new(HashMap::new()));
    let visited_tracker = Arc::new(Mutex::new(ExplorationTracker::new()));
    let exit_position = Arc::new(Mutex::new(None));
    let labyrinth_map: Arc<Mutex<HashMap<(i32, i32), RadarCell>>> = Arc::new(Mutex::new(HashMap::new()));

    // 🧑‍🤝‍🧑 Création des threads des joueurs
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


/// Coordonne les actions du jeu en affichant les actions des joueurs.
///
/// Cette fonction :
/// - Attend que les joueurs envoient leurs actions.
/// - Affiche chaque action reçue.
/// - Termine le jeu une fois que tous les joueurs ont terminé.
///
/// # Paramètres
/// - `rx`: Récepteur pour écouter les actions des joueurs.
/// - `player_count`: Nombre total de joueurs.
///
/// # Exemple
/// ```no_run
/// use std::sync::mpsc::channel;
/// use ma_lib::game_coordinator;
/// use common::message::actiondata::PlayerAction;
///
/// let (tx, rx) = channel();
/// game_coordinator(rx, 2);
/// ```
fn game_coordinator(rx: Receiver<PlayerAction>, player_count: u32) {
    let active_players = player_count;

    while active_players > 0 {
        if let Ok(action) = rx.recv() {
            println!("Joueur {} action: {:?}", action.player_id, action.action);
        }
    }

    println!("Tous les joueurs ont terminé");
}