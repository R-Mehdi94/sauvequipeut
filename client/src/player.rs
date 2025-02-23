use std::collections::HashMap;
use crate::utils::connect_to_server;
use common::message::actiondata::{ActionData, PlayerAction};
use common::message::relativedirection::RelativeDirection;
use std::net::TcpStream;
use common::message::{Message, MessageData};
use common::state::ClientState;
use common::utils::utils::{build_message, handle_response, receive_response, send_message};
use rand::{Rng};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use log::{info, warn};
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use common::message::hintdata::HintData;
use common::message::message::ActionError;
use crate::challenge::{handle_challenge, TeamSecrets};
use crate::decrypte::{decode_and_format, DecodedView};

pub struct Player {
    pub name: String,
    pub registration_token: String,
    pub stream: TcpStream,
}

fn extract_radar_from_response(response: &Message) -> Option<String> {
    match response {
        Message::RadarViewResult(encoded_radar) => Some(encoded_radar.clone()),
        _ => None,
    }
}

pub fn handle_player(
    player_id: u32,
    token: String,
    players: &Arc<Mutex<Vec<Player>>>,
    addr: &str,
    port: &str,
    tx: Sender<PlayerAction>,
    radar_view: Arc<Mutex<DecodedView>>,
) {
    let mut stream = connect_to_server(addr, port).unwrap();
    let player_name = format!("Player_{}", player_id);

    let subscribe_message = build_message(MessageData::SubscribePlayer {
        name: player_name.clone(),
        registration_token: token.clone(),
    }).unwrap();

    send_message(&mut stream, &subscribe_message).unwrap();
    handle_response(&mut stream, &mut ClientState::default()).unwrap();

    let player = Player {
        name: player_name.clone(),
        registration_token: token.clone(),
        stream: stream.try_clone().unwrap(),
    };

    {
        let mut players_lock = players.lock().unwrap();
        players_lock.push(player);
    }

    let team_size = players.lock().unwrap().len();
    let team_secrets = Arc::new(TeamSecrets::new(team_size));

    let mut last_failed_direction: Option<RelativeDirection> = None;
    let mut first_move_done = false;
    let mut last_action = RelativeDirection::Right;

    loop {
        if let Ok(response) = receive_response(&mut stream) {
            println!(
                "Réponse du serveur pour le joueur {}: {:?}",
                player_id, response
            );

            match response {
                Message::Challenge(challenge_data) => {
                    println!("🧭 Challenge reçu pour le joueur {}: {:?}", player_id, challenge_data);
                    handle_challenge(player_id, &challenge_data, &Arc::clone(&team_secrets), &mut stream);
                }
                Message::Hint(hint_data) => {
                    println!("🧭 Indice reçu pour le joueur {}: {:?}", player_id, hint_data);
                    if let HintData::Secret(secret_value) = hint_data {
                        println!("🔑 Secret mis à jour pour le joueur {}: {}", player_id, secret_value);
                        team_secrets.update_secret(player_id, secret_value);
                        println!("📊 Secrets actuels: {:?}", team_secrets.secrets.lock().unwrap());
                    }
                }
                Message::RadarViewResult(radar_encoded) => {
                    if let Ok(decoded_radar) = decode_and_format(&radar_encoded) {
                        let radar_data_locked = decoded_radar;

                        let action = if !first_move_done {
                            println!(" Premier déplacement : Direction droite (Right) comme demandé.");
                            first_move_done = true;
                            ActionData::MoveTo(RelativeDirection::Right)
                        } else {
                            let chosen_action = decide_action(&radar_data_locked, &mut last_action);
                            println!(" Action décidée : {:?}", chosen_action);
                            chosen_action
                        };

                        tx.send(PlayerAction {
                            player_id,
                            action: action.clone(),
                        }).unwrap();

                        let send_result = send_message(&mut stream, &Message::Action(action));
                        if let Err(e) = send_result {
                            eprintln!(" Erreur lors de l'envoi du message: {:?}", e);
                            warn!(" Tentative de reconnexion dans 2 secondes...");
                            thread::sleep(Duration::from_secs(2));
                            if let Ok(new_stream) = connect_to_server(addr, port) {
                                stream = new_stream;
                                eprintln!(" Reconnexion réussie.");

                                let resubscribe_message = build_message(MessageData::SubscribePlayer {
                                    name: player_name.clone(),
                                    registration_token: token.clone(),
                                }).unwrap();

                                if let Err(e) = send_message(&mut stream, &resubscribe_message) {
                                    eprintln!(" Échec de la resouscription après reconnexion: {:?}", e);
                                    return;
                                }
                                handle_response(&mut stream, &mut ClientState::default()).unwrap();
                                eprintln!(" Re-souscription réussie après reconnexion.");
                            } else {
                                eprintln!(" Échec de la reconnexion. Arrêt du joueur.");
                                return;
                            }
                        }
                    }
                }
                Message::ActionError(error) => {
                    println!("️ Erreur d'action pour le joueur {}: {:?}", player_id, error);
                    if let Some(direction) = match error {
                        ActionError::CannotPassThroughWall => Some(RelativeDirection::Right),
                        ActionError::CannotPassThroughOpponent => Some(RelativeDirection::Front),
                        _ => None,
                    } {
                        last_failed_direction = Some(direction);
                    }
                }
                _ => println!(" Réponse non gérée reçue pour le joueur {}: {:?}", player_id, response),
            }
        }
    }
}



    fn decode_passage(value: u32) -> bool {
        value == 1
    }
fn decide_action(radar_data: &DecodedView, last_action: &mut RelativeDirection) -> ActionData {
    println!(
        "🔍 Analyse détaillée du radar:"
    );

    // Vérification des passages dans chaque direction
    let front_open = DecodedView::is_passage_open(radar_data.get_horizontal_passage(1));
    let right_open = DecodedView::is_passage_open(radar_data.get_vertical_passage(1));
    let left_open = DecodedView::is_passage_open(radar_data.get_vertical_passage(0));
    let back_open = DecodedView::is_passage_open(radar_data.get_horizontal_passage(2));



     if radar_data.is_goal_nearby() {
        println!(" Objectif détecté à proximité!");
    }

     let mut possible_moves = Vec::new();

     if front_open {
        possible_moves.push(RelativeDirection::Front);
    }
    if right_open {
        possible_moves.push(RelativeDirection::Right);
    }
    if left_open {
        possible_moves.push(RelativeDirection::Left);
    }
    if back_open {
        possible_moves.push(RelativeDirection::Back);
    }

     if possible_moves.is_empty() {
         // Rotation systématique
        *last_action = match *last_action {
            RelativeDirection::Front => RelativeDirection::Right,
            RelativeDirection::Right => RelativeDirection::Back,
            RelativeDirection::Back => RelativeDirection::Left,
            RelativeDirection::Left => RelativeDirection::Front,
        };
        return ActionData::MoveTo(*last_action);
    }

    // Essayer d'éviter de revenir sur ses pas si possible
    let best_move = if possible_moves.len() > 1 {
        match *last_action {
            RelativeDirection::Front => {
                if back_open && possible_moves.len() == 1 && possible_moves[0] == RelativeDirection::Back {
                    &RelativeDirection::Back
                } else {
                    possible_moves.iter().find(|&&dir| dir != RelativeDirection::Back).unwrap_or(&possible_moves[0])
                }
            },
            _ => &possible_moves[0]
        }
    } else {
        &possible_moves[0]
    };

    println!(" Direction choisie: {:?}", best_move);
    *last_action = *best_move;
    ActionData::MoveTo(*best_move)
}