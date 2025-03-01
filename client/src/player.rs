use std::cmp::PartialEq;
use std::collections::{HashMap, HashSet};
use crate::utils::connect_to_server;
use common::message::actiondata::{ActionData, PlayerAction};
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
use crate::decrypte::{decode_and_format, DecodedView, RadarCell};
use crate::exploration_tracker::ExplorationTracker;
use crate::hint::{direction_from_angle, direction_from_grid_size, handle_hint};
use crate::player_memory::PlayerMemory;
use crate::radar_view::{choose_accessible_direction, compute_absolute_position, decide_action, detect_near_border, find_path_to_exit, follower_choose_action, leader_choose_action, send_action, simulate_movement, update_player_position};

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
    hint_received: Arc<Mutex<bool>>,
    last_radar_view: Arc<Mutex<Option<DecodedView>>>,
    player_memories: Arc<Mutex<HashMap<u32, PlayerMemory>>>,
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
                            &Arc::clone(&shared_grid_size),
                            &Arc::clone(&hint_received)

                        );
                    }
                }

                Message::RadarViewResult(radar_encoded) => {
                    if let Ok(decoded_radar) = decode_and_format(&radar_encoded) {
                        println!(" [DEBUG] Décode radar réussi pour le joueur {}", player_id);
                        let mut last_radar_lock = last_radar_view.lock().unwrap();
                        *last_radar_lock = Some(decoded_radar.clone());
                        drop(last_radar_lock);
                        let mut position_map = position_tracker.lock().unwrap();  // 🔒 Verrouillage du mutex
                        let player_position = position_map.entry(player_id).or_insert((0, 0));
                        let current_position = *player_position;

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
                        drop(exit_lock);


                        let num_connected_players = players.lock().unwrap().len();

                        let leader_exists = {
                            let players_locked = players.lock().unwrap();
                            let leader_locked = leader_id.lock().unwrap();

                            match *leader_locked {
                                Some(leader_index) => {
                                    if players_locked.get(leader_index as usize).is_some() {
                                        println!("✅ [INFO] Leader {} est toujours actif.", leader_index);
                                        true
                                    } else {
                                        println!("⚠️ [INFO] Leader {} a quitté la partie ou a atteint la sortie !", leader_index);
                                        false
                                    }
                                }
                                None => false,
                            }
                        };
 /*
                        let action = if num_connected_players == 1 {
                            println!("🧑‍🚀 Joueur {} est seul et suit sa propre stratégie.", player_id);

                             if let Some(exit_pos) = *exit_position.lock().unwrap() {
                                if let Some(direction) = find_path_to_exit(player_id, &position_tracker.lock().unwrap(), exit_pos) {
                                    println!("🚀 [SORTIE] Joueur {} se dirige vers {:?}", player_id, direction);
                                    last_direction = Some(direction);
                                      ActionData::MoveTo(direction);
                                } else {
                                    println!("❌ [SORTIE] Impossible de déterminer un chemin vers la sortie !");
                                }
                            }


                            else if let Some((cols, rows)) = grid_size {


                                    println!("🗺️ [INFO] Joueur {} détecte un labyrinthe de {}x{}.", player_id, cols, rows);

                                    let direction_priority = direction_from_grid_size(grid_size);
                                    println!("➡️ [GRID PRIORITY] Direction suggérée : {:?}", direction_priority);

                                    if let Some(direction) = choose_accessible_direction(&decoded_radar, direction_priority) {
                                        if let Some(new_position) = simulate_movement(player_id, direction, &position_tracker.lock().unwrap()) {
                                            let visit_count = visited_map.get(&new_position).cloned().unwrap_or(0);

                                            if visit_count < 3 {
                                                println!("✅ [GRID] Direction {:?} choisie avec {} visites.", direction, visit_count);
                                                  ActionData::MoveTo(direction);
                                            }
                                        }
                                    }
                                    println!("🧱 [GRID FAIL] Aucune bonne direction trouvée avec GridSize.");
                                }
                                println!("🧱 [GRID FAIL] Aucune bonne direction trouvée avec GridSize.");



                            if let Some(angle) = compass_angle {
                                let direction_priority = direction_from_angle(angle);
                                if let Some(direction) = choose_accessible_direction(&decoded_radar, direction_priority) {
                                    println!("🧭 [BOUSSOLE] Joueur {} suit la direction {:?}", player_id, direction);
                                      ActionData::MoveTo(direction);
                                }
                            }


                            println!("🌍 [DERNIER RECOURS] Joueur {} explore une nouvelle direction.", player_id);
                            decide_action(&decoded_radar)
                        }
*/



                        let action =   if leader_exists && Some(player_id) == *leader_id.lock().unwrap() {
                            println!("🟢 [LEADER] Joueur {} agit en tant que leader.", player_id);
                            leader_choose_action(
                                player_id,
                                &decoded_radar,
                                grid_size,
                                compass_angle,
                                &visited_map,
                                &position_tracker.lock().unwrap(),
                                &exit_position,
                                &player_memories
                            )
                        } else if leader_exists  && num_connected_players>1{
                            println!("🔵 [FOLLOWER] Joueur {} suit le leader.", player_id);
                            follower_choose_action(
                                player_id,
                                &decoded_radar,
                                &shared_leader_action
                            )
                        } else {
                            println!("⚠️ [INFO] Aucun leader actif, Joueur {} agit individuellement.", player_id);
                            decide_action(&decoded_radar)
                        };




                        update_player_position(player_id, position_map.get_mut(&player_id).unwrap(), &action);
                        let new_position = *position_map.get(&player_id).unwrap(); // Obtenir la nouvelle position

                        let mut tracker = visited_tracker.lock().unwrap();
                        tracker.mark_position(new_position); // Ajoute la position visitée

                        println!(
                            " [POSITION] Joueur {} est en {:?}, visité {} fois",
                            player_id,
                            new_position,
                            tracker.visited_positions.get(&new_position).unwrap_or(&0)
                        );



 /*
                        println!("🗺️ [DEBUG] Carte mémorisée :");
                        let map_lock = labyrinth_map.lock().unwrap();
                        for (position, cell) in map_lock.iter() {
                            println!("📍 Position: {:?} → Cellule: {:?}", position, cell);
                        }
                        println!("🗺️ [DEBUG] Fin de l'affichage de la carte.");
*/
                        send_action(player_id, action, &tx, &mut stream);

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
                            println!("🚧 [ERREUR] Joueur {} a tenté de passer à travers un mur !", player_id);

                         /*   // 🔄 Choisir une direction aléatoire (par défaut)
                            let mut rng = rand::thread_rng();
                            let random_direction = match rng.gen_range(0..4) {
                                0 => RelativeDirection::Front,
                                1 => RelativeDirection::Right,
                                2 => RelativeDirection::Left,
                                _ => RelativeDirection::Back,
                            };

                            println!("🔀 [RANDOM] Joueur {} essaie une autre direction : {:?}", player_id, random_direction);


                            if let Some(last_radar) = last_radar_view.lock().unwrap().as_ref() {
                                if let Some(direction) = choose_accessible_direction(last_radar, vec![random_direction]) {
                                    println!("✅ [NOUVELLE DIRECTION] Joueur {} se dirige vers {:?}", player_id, direction);
                                  let action = ActionData::MoveTo(direction);

                                    let mut position_map = position_tracker.lock().unwrap();
                                    update_player_position(player_id, position_map.get_mut(&player_id).unwrap(), &action);


                                    send_action(player_id, action, &tx, &mut stream);
                                } else {
                                    println!("⚠️ [ERREUR] Aucune direction accessible trouvée, tentative avec un mouvement aléatoire.");
                                    let action = ActionData::MoveTo(random_direction);

                                    let mut position_map = position_tracker.lock().unwrap();
                                    update_player_position(player_id, position_map.get_mut(&player_id).unwrap(), &action);
                                    send_action(player_id, action, &tx, &mut stream);
                                }
                            }

*/
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






