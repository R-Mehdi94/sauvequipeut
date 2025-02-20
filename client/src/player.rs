use crate::utils::connect_to_server;
use common::message::actiondata::{ActionData, PlayerAction};
use common::message::relativedirection::RelativeDirection;
use std::net::TcpStream;

use common::message::{Message, MessageData};
use common::state::ClientState;
use common::utils::utils::{build_message, handle_response, receive_response, send_message};
use rand::Rng;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

pub struct Player {
    pub name: String,
    pub registration_token: String,
    pub stream: TcpStream,
}

pub fn handle_player(
    player_id: u32,
    token: String,
    players: &Arc<Mutex<Vec<Player>>>,
    addr: &str,
    port: &str,
    tx: Sender<PlayerAction>,
) {
    let mut stream = connect_to_server(addr, port).unwrap();
    let player_name = format!("Player_{}", player_id);

    let subscribe_message = build_message(MessageData::SubscribePlayer {
        name: player_name.clone(),
        registration_token: token.clone(),
    })
    .unwrap();

    send_message(&mut stream, &subscribe_message).unwrap();
    handle_response(&mut stream, &mut ClientState::default()).unwrap();

    let player = Player {
        name: player_name,
        registration_token: token,
        stream: stream.try_clone().unwrap(),
    };

    {
        let mut players_lock = players.lock().unwrap();
        players_lock.push(player);
    }

    loop {
        let action = decide_action();
        tx.send(PlayerAction { player_id, action }).unwrap();
        let action_for_send = decide_action(); // Créer une nouvelle action car on peut pas clone action

        send_message(&mut stream, &Message::Action(action_for_send)).unwrap();

        // Recevoir et traiter la réponse du serveur
        if let Ok(response) = receive_response(&mut stream) {
            // Traiter la réponse ici
            println!(
                "Réponse du serveur pour le joueur {}: {:?}",
                player_id, response
            );
        }
    }
}

fn decide_action() -> ActionData {
    let mut rng = rand::rng();
    match rng.random_range(0..4) {
        0 => ActionData::MoveTo(RelativeDirection::Front),
        1 => ActionData::MoveTo(RelativeDirection::Back),
        2 => ActionData::MoveTo(RelativeDirection::Left),
        _ => ActionData::MoveTo(RelativeDirection::Right),
    }
}
