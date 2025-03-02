use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use crate::utils::connect_to_server;
use common::message::actiondata::{ActionData, PlayerAction};
use common::message::relativedirection::RelativeDirection;
use std::net::TcpStream;
use common::message::{Message, MessageData};
use common::state::ClientState;
use common::utils::utils::{build_message, handle_response, receive_response, send_message};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use common::message::challengedata::ChallengeData;
use common::message::hintdata::HintData;
use common::message::message::ActionError;
use crate::challenge::{handle_challenge, TeamSecrets};
use crate::decrypte::{decode_and_format, RadarCell};
use crate::exploration_tracker::ExplorationTracker;
use crate::hint::{ handle_hint};
use crate::radar_view::{ choose_least_visited_direction, compute_absolute_position, decide_action,  follower_choose_action, leader_choose_action, send_action, update_player_position};

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
    position_tracker: Arc<Mutex<HashMap<u32, (i32, i32)>>>,
    visited_tracker: Arc<Mutex<ExplorationTracker>>,
    exit_position: Arc<Mutex<Option<(i32, i32)>>>,
    labyrinth_map: Arc<Mutex<HashMap<(i32, i32), RadarCell>>>,
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
                    if let HintData::Secret(secret_value) = hint_data {
                        team_secrets.update_secret(player_id, secret_value);
                    }
                    else{
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
                        println!(" [DEBUG] Décode radar réussi pour le joueur {}", player_id);

                        let mut position_map = position_tracker.lock().unwrap();
                        let player_position = position_map.entry(player_id).or_insert((0, 0));
                        let current_position = *player_position;
                        drop(position_map);

                        let mut visited_map = visited_tracker.lock().unwrap();

                        let grid_size = *shared_grid_size.lock().unwrap();
                        let compass_angle = *shared_compass.lock().unwrap();

                        let mut map_lock = labyrinth_map.lock().unwrap();
                        for (index, cell) in decoded_radar.cells.iter().enumerate() {
                            let absolute_position = compute_absolute_position(current_position, index);
                            map_lock.insert(absolute_position, cell.clone());
                        }
                        drop(map_lock);
                        let mut exit_lock = exit_position.lock().unwrap();
                        for (index, cell) in decoded_radar.cells.iter().enumerate() {
                            if *cell == RadarCell::Exit {
                                let exit_pos = compute_absolute_position(current_position, index);
                                *exit_lock = Some(exit_pos);
                                println!("🏁 [SORTIE DÉTECTÉE] Joueur {} a trouvé la sortie en {:?}", player_id, exit_pos);
                            }
                        }
                        let num_connected_players = players.lock().unwrap().len();

                        drop(exit_lock);
                        let is_leader = {
                            let mut leader_locked = leader_id.lock().unwrap();


                            if let Some(id) = *leader_locked {
                                id == player_id
                            } else if num_connected_players == 1 {
                                *leader_locked = None;
                                false
                            }
                            else {

                                println!("👑 [INFO] Aucun leader défini, on élit un leader : {}", player_id);
                                *leader_locked = Some(player_id);
                                true
                            }

                        };

                        let action =
                            if is_leader {
                                println!("🟢 [LEADER] Choix de direction pour le leader.");
                                leader_choose_action(player_id, &decoded_radar, grid_size, compass_angle,   &mut visited_map, &position_tracker.lock().unwrap(),&exit_position)
                            }
                            else {
                                let leader_option = leader_id.lock().unwrap().clone();
                                if let Some(leader) = leader_option {
                                    println!("🔵 [FOLLOWER] Suivi du leader {}.", leader);
                                    follower_choose_action(player_id, &decoded_radar, &shared_leader_action)
                                } else if num_connected_players == 1 {
                                    let current_position = {
                                        let position_map = position_tracker.lock().unwrap();
                                        *position_map.get(&player_id).unwrap()
                                    };

                                    if visited_map.is_recently_visited(current_position) {
                                        println!("🔄 [ALERTE] Joueur {} est coincé dans une boucle ! Recherche d'un nouveau chemin...", player_id);

                                        let action = choose_least_visited_direction(
                                            player_id,
                                            &decoded_radar,
                                            &mut visited_map,
                                            &position_tracker.lock().unwrap()
                                        );
                                        action
                                    }
                                    else {
                                        println!("⚙️ [INFO] Joueur {} explore normalement.", player_id);
                                        decide_action(&decoded_radar)
                                    }

                                }
                                else {
                                    println!("⚠️ [AUTONOME] Aucun leader défini, stratégie plombier.");
                                    decide_action(&decoded_radar)
                                }
                            };

                        let mut position_map = position_tracker.lock().unwrap();
                        update_player_position(player_id, position_map.get_mut(&player_id).unwrap(), &action, &mut visited_map);
                        println!(" [ENVOI] Action du joueur {} : {:?}", player_id, action);
                        send_action(player_id, action, &tx, &mut stream);

                        {
                            let num_players = players.lock().unwrap().len();
                            let mut leader = leader_id.lock().unwrap();

                            if num_players == 1 && leader.map_or(false, |id| id == player_id) {
                                println!("👑 [LEADER RESET] Joueur {} est seul, réinitialisation du leader.", player_id);
                                *leader = None;
                            }


                        }

                        println!(" [DEBUG] Fin de traitement du radar pour le joueur {}", player_id);
                    } else {
                        println!(" [ERROR] Échec du décodage radar pour le joueur {}", player_id);
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
