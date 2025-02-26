use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
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
use common::message::challengedata::ChallengeData;
use common::message::hintdata::HintData;
use common::message::message::ActionError;
use crate::challenge::{handle_challenge, TeamSecrets};
use crate::decrypte::{decode_and_format, exemple, DecodedView, RadarCell};
use crate::hint::{direction_from_angle, direction_from_grid_size, handle_hint};
use crate::Position::Position;

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
    team_secrets: Arc<TeamSecrets>,
    shared_compass :Arc<Mutex<Option<f32>>>,
    leader_id:Arc<Mutex<Option<u32>>>,
    shared_leader_action: Arc<Mutex<Option<ActionData>>>,
    shared_grid_size: Arc<Mutex<Option<(u32, u32)>>>,
    player_position: Arc<Mutex<Position>>,
    explored_cells: Arc<Mutex<HashSet<(i32, i32)>>>,


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
    //exemple ();


    let mut last_challenge:Option<ChallengeData>=None;
    loop {
        if let Ok(response) = receive_response(&mut stream) {
            println!(
                "Réponse du serveur pour le joueur {}: {:?}",
                player_id, response
            );

            match response {
                Message::Challenge(challenge_data) => {
                    println!(" Challenge reçu pour le joueur {}: {:?}", player_id, challenge_data);
                    last_challenge = Some(challenge_data.clone());
                    handle_challenge(player_id, &challenge_data, &Arc::clone(&team_secrets), &mut stream);

                }

                Message::Hint(hint_data) => {
                    println!(" Indice reçu pour le joueur {}: {:?}", player_id, hint_data);
                    if let HintData::Secret(secret_value) = hint_data {
                        println!(" Secret mis à jour pour le joueur {}: {}", player_id, secret_value);
                        team_secrets.update_secret(player_id, secret_value);
                        println!(" Secrets actuels: {:?}", team_secrets.secrets.lock().unwrap());
                    }
                    else{
                        println!("📩 Indice reçu pour le joueur {}: {:?}", player_id, hint_data);
                        handle_hint(
                            player_id,
                            &hint_data,
                            &Arc::clone(&shared_compass),
                            &Arc::clone(&leader_id) ,
                            &Arc::clone(&shared_grid_size)
                        );
                    }
                }

                Message::RadarViewResult(radar_encoded) => {
                    if let Ok(decoded_radar) = decode_and_format(&radar_encoded) {
                        let radar_data_locked = decoded_radar;
                        {
                            let mut explored = explored_cells.lock().unwrap();
                           // explored.insert(radar_data_locked.current_position);
                        }
                        let grid_size = *shared_grid_size.lock().unwrap();
                        let compass_angle = *shared_compass.lock().unwrap();

                        let is_leader = {
                            let mut leader_locked = leader_id.lock().unwrap();
                            if leader_locked.is_none() {
                                println!("👑 [LEADER] Le joueur {} devient temporairement leader (GridSize).", player_id);
                                *leader_locked = Some(player_id);
                                true
                            } else {
                                leader_locked.map_or(false, |id| id == player_id)
                            }
                        };


                        if is_leader {
                            let action = if let Some((cols, rows)) = grid_size {
                                println!("🗺️ [LEADER] Taille labyrinthe : {} colonnes x {} lignes.", cols, rows);
                                let direction_priority = direction_from_grid_size(grid_size);

                                if let Some(direction) = choose_accessible_direction(&radar_data_locked, direction_priority) {
                                    ActionData::MoveTo(direction)
                                } else {
                                    println!("🧱 [FALLBACK] GridSize échoué ➔ Tentative avec la boussole.");

                                    if let Some(angle) = compass_angle {
                                        println!("🧭 [LEADER] Utilisation de la boussole : {:.2}°", angle);
                                        let direction_priority = direction_from_angle(angle);
                                        choose_accessible_direction(&radar_data_locked, direction_priority)
                                            .map(ActionData::MoveTo)
                                            .unwrap_or_else(|| {
                                                println!("🧱 [FALLBACK FINAL] Boussole échouée ➔ Stratégie plombier.");
                                                decide_action(&radar_data_locked)
                                            })
                                    } else {
                                        println!("🧭 [INFO] Boussole non disponible ➔ Stratégie plombier.");
                                        decide_action(&radar_data_locked)
                                    }
                                }
                            } else if let Some(angle) = compass_angle {
                                println!("🧭 [LEADER] Boussole disponible (sans GridSize) : {:.2}°", angle);
                                let direction_priority = direction_from_angle(angle);
                                choose_accessible_direction(&radar_data_locked, direction_priority)
                                    .map(ActionData::MoveTo)
                                    .unwrap_or_else(|| {
                                        println!("🧱 [FALLBACK FINAL] Boussole échouée ➔ Stratégie plombier.");
                                        decide_action(&radar_data_locked)
                                    })
                            } else {
                                println!("⚙️ [INFO] Aucune information (GridSize/Compass) ➔ Stratégie plombier.");
                                decide_action(&radar_data_locked)
                            };

                            {
                                let mut leader_action_locked = shared_leader_action.lock().unwrap();
                                *leader_action_locked = Some(action.clone());
                            }

                            tx.send(PlayerAction {
                                player_id,
                                action: action.clone(),
                            }).unwrap();

                            let send_result = send_message(&mut stream, &Message::Action(action));
                            if let Err(e) = send_result {
                                warn!("🔄 Tentative de reconnexion dans 2 secondes...");
                                thread::sleep(Duration::from_secs(2));
                            }
                        }

                        else {
                            let current_leader = leader_id.lock().unwrap();
                            if current_leader.is_none() {

                                println!("🤖 Joueur {} : Pas de leader actuel, exploration avec stratégie plombier.", player_id);
                                let action = decide_action(&radar_data_locked);

                                tx.send(PlayerAction {
                                    player_id,
                                    action: action.clone(),
                                }).unwrap();

                                let send_result = send_message(&mut stream, &Message::Action(action));
                                if let Err(e) = send_result {
                                    warn!("🔄 Tentative de reconnexion dans 2 secondes...");
                                    thread::sleep(Duration::from_secs(2));
                                }
                            } else {

                                println!("🤝 Joueur {} attend l'action du leader.", player_id);

                                let leader_action = {
                                    let action_locked = shared_leader_action.lock().unwrap();
                                    action_locked.clone()
                                };
                                if let Some(action) = leader_action {
                                    println!("🤝 Joueur {} essaye de suivre l'action du leader : {:?}", player_id, action);

                                    let accessible_direction = follow_leader_direction(&radar_data_locked, match action {
                                        ActionData::MoveTo(dir) => dir,
                                        _ => RelativeDirection::Front,
                                    });

                                    if let Some(adapted_direction) = accessible_direction {
                                        println!("🤝 Joueur {} suit finalement la direction : {:?}", player_id, adapted_direction);

                                        let adapted_action = ActionData::MoveTo(adapted_direction);
                                        tx.send(PlayerAction {
                                            player_id,
                                            action: adapted_action.clone(),
                                        }).unwrap();

                                        let send_result = send_message(&mut stream, &Message::Action(adapted_action));
                                        if let Err(e) = send_result {
                                            warn!("🔄 Tentative de reconnexion dans 2 secondes...");
                                            thread::sleep(Duration::from_secs(2));
                                        }
                                    } else {
                                        println!("🤝 Joueur {} ne peut pas suivre le leader (toutes directions bloquées). Attente...", player_id);
                                        thread::sleep(Duration::from_millis(500)); // Attente courte
                                    }
                                }

                            }
                        }

                    }
                }

                Message::ActionError(error) => {
                    match error {
                        ActionError::InvalidChallengeSolution=> {
                            println!(" [INVALID] Le serveur a rejeté la solution. 🔄 Recalcul immédiat...: {:?}", error);

                            if let Some(challenge) = &last_challenge {
                                handle_challenge(player_id, challenge, &Arc::clone(&team_secrets), &mut stream);
                            } else {
                                println!("⚠️ Aucun challenge précédent trouvé pour recalculer.");
                            }
                        }
                        ActionError::CannotPassThroughWall => {
                            println!("🚧 [MUR] Impossible de passer à travers le mur. 🚫 Changer de direction !: {:?}", error );
                        }
                        _ => {
                            println!("⚠️ [ERREUR NON GÉRÉE] : {:?}", error);
                        }
                    }
                }

                _ => println!("🔍 Réponse non gérée pour le joueur {}: {:?}", player_id, response),



            }
        }
    }

}



fn decode_passage(value: u32) -> bool {
    value == 1
}



pub fn is_passage_open(passage: u32, bit_index: usize) -> bool {

    let corrected_index = 3- bit_index;
    let bits = (passage >> (corrected_index * 2)) & 0b11;

    println!(
        "🔎 Vérification passage: bits = {:02b}, bit_index = {}, corrected_index = {}",
        bits, bit_index, corrected_index
    );

    match bits {
        0b01 => {
            println!(" Passage ouvert !");
            true
        }
        0b00 | 0b10 => {
            println!(" Passage fermé !");
            false
        }
        _ => {
            println!("⚠Valeur inattendue !");
            false
        }
    }
}
pub fn choose_accessible_direction(radar: &DecodedView, directions: Vec<RelativeDirection>) -> Option<RelativeDirection> {
    for direction in directions {
        let accessible = match direction {
            RelativeDirection::Front => {
                let front_cell = &radar.cells[1];
                *front_cell == RadarCell::Open && is_passage_open(radar.get_horizontal_passage(1), 2)
            }
            RelativeDirection::Right => {
                let right_cell = &radar.cells[5];
                *right_cell == RadarCell::Open && is_passage_open(radar.get_vertical_passage(1), 2)
            }
            RelativeDirection::Left => {
                let left_cell = &radar.cells[3];
                *left_cell == RadarCell::Open && is_passage_open(radar.get_vertical_passage(1), 1)
            }
            RelativeDirection::Back => {
                let back_cell = &radar.cells[7];
                *back_cell == RadarCell::Open && is_passage_open(radar.get_horizontal_passage(2), 2)
            }
        };

        if accessible {
            println!("✅ [ACCESSIBLE] Direction accessible : {:?}", direction);
            return Some(direction);
        } else {
            println!("🚫 [BLOQUÉ] Direction bloquée : {:?}", direction);
        }
    }
    println!("⚠️ [INFO] Aucune direction accessible.");
    None
}

pub fn follow_leader_direction(radar: &DecodedView, leader_direction: RelativeDirection) -> Option<RelativeDirection> {
     let direction_priority = match leader_direction {
        RelativeDirection::Front => vec![RelativeDirection::Front, RelativeDirection::Right, RelativeDirection::Left, RelativeDirection::Back],
        RelativeDirection::Right => vec![RelativeDirection::Right, RelativeDirection::Front, RelativeDirection::Back, RelativeDirection::Left],
        RelativeDirection::Left => vec![RelativeDirection::Left, RelativeDirection::Front, RelativeDirection::Back, RelativeDirection::Right],
        RelativeDirection::Back => vec![RelativeDirection::Back, RelativeDirection::Left, RelativeDirection::Right, RelativeDirection::Front],
    };

    choose_accessible_direction(radar, direction_priority)
}


pub fn decide_action(radar: &DecodedView) -> ActionData {
    let front_cell = &radar.cells[1];
    let right_cell = &radar.cells[5];
    let left_cell = &radar.cells[3];


    let right_open = *right_cell == RadarCell::Open
        && is_passage_open(radar.get_vertical_passage(1), 2);

    let front_open = *front_cell == RadarCell::Open
        && is_passage_open(radar.get_horizontal_passage(1), 2);

    let left_open = *left_cell == RadarCell::Open
        && is_passage_open(radar.get_vertical_passage(1), 1);

    if right_open {
         ActionData::MoveTo(RelativeDirection::Right)
    } else if front_open {
         ActionData::MoveTo(RelativeDirection::Front)
    } else if left_open {
         ActionData::MoveTo(RelativeDirection::Left)
    } else  {
         ActionData::MoveTo(RelativeDirection::Back)
    }
}


