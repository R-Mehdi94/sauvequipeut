use std::collections::HashMap;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use log::warn;
use common::message::actiondata::{ActionData, PlayerAction};
use common::message::Message;
use common::message::relativedirection::RelativeDirection;
use common::utils::utils::send_message;
use crate::decrypte::{decode_and_format, is_passage_open, DecodedView, RadarCell};
use crate::hint::{direction_from_angle, direction_from_grid_size};


pub fn handle_radar_view(
    player_id: u32,
    radar_encoded: String,
    shared_grid_size: Arc<Mutex<Option<(u32, u32)>>>,
    shared_compass: Arc<Mutex<Option<f32>>>,
    leader_id: Arc<Mutex<Option<u32>>>,
    shared_leader_action: Arc<Mutex<Option<ActionData>>>,
    tx: Sender<PlayerAction>,
    mut stream: TcpStream,
    position_tracker: Arc<Mutex<HashMap<u32, (i32, i32)>>>,
    visited_tracker: Arc<Mutex<HashMap<(i32, i32), usize>>>,
) {
    if let Ok(decoded_radar) = decode_and_format(&radar_encoded) {
        let mut position_map = position_tracker.lock().unwrap();


        let player_position = position_map.entry(player_id).or_insert((0, 0));
        let current_position = *player_position;


        let mut visited_map = visited_tracker.lock().unwrap();
        let visit_count = visited_map.entry(current_position).or_insert(0);
        *visit_count += 1;

        println!("📍 [POSITION] Joueur {} est en {:?}, visité {} fois", player_id, current_position, *visit_count);

        let grid_size = *shared_grid_size.lock().unwrap();
        let compass_angle = *shared_compass.lock().unwrap();

        let is_leader = determine_leader(player_id, &leader_id);

        drop(position_map);

        let action = if is_leader {
            leader_choose_action(player_id, &decoded_radar, grid_size, compass_angle, &visited_map, &position_tracker.lock().unwrap())
        } else {
            let leader_exists = leader_id.lock().unwrap().is_some();
            if leader_exists {
                follower_choose_action(player_id, &decoded_radar, &shared_leader_action)
            } else {
                println!("⚠️ [INFO] Aucun leader encore défini, utilisation de la stratégie plombier.");
                decide_action(&decoded_radar)
            }
        };

        // 📌 Reprendre le lock pour mettre à jour la position
        let mut position_map = position_tracker.lock().unwrap();
        update_player_position(player_id, position_map.get_mut(&player_id).unwrap(), &action);

        send_action(player_id, action, &tx, &mut stream);

    }
}



pub fn calculate_position(player_id: u32, radar_data: &DecodedView) -> (i32, i32) {

    let x = player_id as i32 % 10;
    let y = (player_id as i32 / 10) % 10;
    (x, y)
}


pub fn determine_leader(player_id: u32, leader_id: &Arc<Mutex<Option<u32>>>) -> bool {
    let mut leader_locked = leader_id.lock().unwrap();
    if leader_locked.is_none() {
        println!("👑 [LEADER] Le joueur {} devient temporairement leader.", player_id);
        *leader_locked = Some(player_id);
        true
    } else {
        leader_locked.map_or(false, |id| id == player_id)
    }
}
pub fn choose_least_visited_direction(
    player_id: u32,
    radar_data: &DecodedView,
    tracker: &HashMap<(i32, i32), usize>,
    position_tracker: &HashMap<u32, (i32, i32)>,
) -> ActionData {
    let directions = vec![
        RelativeDirection::Front,
        RelativeDirection::Right,
        RelativeDirection::Left,
        RelativeDirection::Back,
    ];

    let mut best_direction = None;
    let mut lowest_visits = usize::MAX;

    for direction in directions {
        if let Some(new_position) = simulate_movement(player_id, direction, position_tracker) {
            let visit_count = tracker.get(&new_position).cloned().unwrap_or(0);

            if visit_count < lowest_visits {
                lowest_visits = visit_count;
                best_direction = Some(direction);
            }
        }
    }

    if let Some(direction) = best_direction {
        println!(
            "✅ [DIRECTION] Joueur {} choisit {:?} avec {} visites",
            player_id, direction, lowest_visits
        );
        return ActionData::MoveTo(direction);
    }

    println!("⚠️ [DIRECTION] Aucune direction optimale trouvée, retour à la stratégie plombier.");
    decide_action(radar_data)
}

pub fn simulate_movement(
    player_id: u32,
    direction: RelativeDirection,
    position_tracker: &HashMap<u32, (i32, i32)>,
) -> Option<(i32, i32)> {

    let (x, y) = match position_tracker.get(&player_id) {
        Some(pos) => *pos,
        None => return None,
    };

    let new_position = match direction {
        RelativeDirection::Front => (x, y - 1),
        RelativeDirection::Right => (x + 1, y),
        RelativeDirection::Left => (x - 1, y),
        RelativeDirection::Back => (x, y + 1),
    };

    Some(new_position)
}



  pub fn leader_choose_action(
    player_id: u32,
    radar_data: &DecodedView,
    grid_size: Option<(u32, u32)>,
    compass_angle: Option<f32>,
    tracker: &HashMap<(i32, i32), usize>,
    position_tracker: &HashMap<u32, (i32, i32)>,
) -> ActionData {
    if let Some((cols, rows)) = grid_size {
        println!("🗺️ [LEADER] Taille labyrinthe : {} colonnes x {} lignes.", cols, rows);
        let direction_priority = direction_from_grid_size(grid_size);

        if let Some(direction) = choose_accessible_direction(radar_data, direction_priority) {
            if let Some(new_position) = simulate_movement(player_id, direction, position_tracker) {
                let visit_count = tracker.get(&new_position).cloned().unwrap_or(0);

                if visit_count < 3 {
                    println!("✅ [LEADER] Direction {:?} choisie avec {} visites.", direction, visit_count);
                    return ActionData::MoveTo(direction);
                } else {
                    println!("⚠️ [LEADER] Trop de visites pour {:?}, on cherche autre chose.", direction);
                }
            }
        }
        println!("🧱 [FALLBACK] GridSize échoué ➔ Tentative avec la boussole.");
    }

    if let Some(angle) = compass_angle {
        println!("🧭 [LEADER] Utilisation de la boussole : {:.2}°", angle);
        let direction_priority = direction_from_angle(angle);

        if let Some(direction) = choose_accessible_direction(radar_data, direction_priority) {
            if let Some(new_position) = simulate_movement(player_id, direction, position_tracker) {
                let visit_count = tracker.get(&new_position).cloned().unwrap_or(0);

                if visit_count < 3 {
                    println!("✅ [LEADER] Direction {:?} choisie via boussole avec {} visites.", direction, visit_count);
                    return ActionData::MoveTo(direction);
                } else {
                    println!("⚠️ [LEADER] Trop de visites pour {:?}, on cherche autre chose.", direction);
                }
            }
        }
        println!("🧱 [FALLBACK FINAL] Boussole échouée ➔ Stratégie plombier.");
    } else {
        println!("⚙️ [INFO] Aucune information ➔ Stratégie plombier.");
    }

    // 🚀 Dernier recours : Choisir la direction la moins visitée
    choose_least_visited_direction(player_id, radar_data, tracker, position_tracker)
}

pub fn save_leader_action(shared_leader_action: &Arc<Mutex<Option<ActionData>>>, action: &ActionData) {
    let mut leader_action_locked = shared_leader_action.lock().unwrap();
    *leader_action_locked = Some(action.clone());
}

pub fn follower_choose_action(
    player_id: u32,
    radar_data: &DecodedView,
    shared_leader_action: &Arc<Mutex<Option<ActionData>>>
) -> ActionData {
    println!("🤝 Joueur {} attend l'action du leader.", player_id);

    let leader_action = {
        let action_locked = shared_leader_action.lock().unwrap();
        action_locked.clone()
    };

    if let Some(action) = leader_action {
        println!("🤝 Joueur {} essaye de suivre l'action du leader : {:?}", player_id, action);
        if let ActionData::MoveTo(direction) = action {
            if let Some(adapted_direction) = follow_leader_direction(radar_data, direction) {
                println!("✅ [SUIVI] Joueur {} suit finalement : {:?}", player_id, adapted_direction);
                return ActionData::MoveTo(adapted_direction);
            }
        }
    }

    println!("🧱 [INFO] Joueur {} ne peut pas suivre ➔ Stratégie plombier.", player_id);
    decide_action(radar_data)
}

pub fn send_action(
    player_id: u32,
    action: ActionData,
    tx: &Sender<PlayerAction>,
    stream: &mut TcpStream
) {
    tx.send(PlayerAction {
        player_id,
        action: action.clone(),
    }).unwrap();

    if let Err(e) = send_message(stream, &Message::Action(action)) {
        warn!("🔄 Tentative de reconnexion dans 2 secondes...");
        thread::sleep(Duration::from_secs(2));
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
    println!("radar recu {:?}",radar);

    println!(
        "🔍 [DEBUG] Vérification des cellules : Front={:?}, Right={:?}, Left={:?}",
        front_cell, right_cell, left_cell
    );

    let right_open = *right_cell == RadarCell::Open
        && is_passage_open(radar.get_vertical_passage(1), 2);

    let front_open = *front_cell == RadarCell::Open
        && is_passage_open(radar.get_horizontal_passage(1), 2);

    let left_open = *left_cell == RadarCell::Open
        && is_passage_open(radar.get_vertical_passage(1), 1);
    println!("Bits horizontaux (ligne 1): {:06b}", radar.get_horizontal_passage(1));
    println!("Bits verticaux (colonne 1): {:08b}", radar.get_vertical_passage(1));
    println!("Bits verticaux (colonne 1): {:08b}", radar.get_vertical_passage(2));

    if right_open {

        ActionData::MoveTo(RelativeDirection::Right)
    } else if front_open {

        ActionData::MoveTo(RelativeDirection::Front)
    } else if left_open {

        ActionData::MoveTo(RelativeDirection::Left)
    } else {

        ActionData::MoveTo(RelativeDirection::Back)
    }
}


pub fn update_player_position(
    player_id: u32,
    player_position: &mut (i32, i32),
    action: &ActionData
) {
    if let ActionData::MoveTo(direction) = action {
        match direction {
            RelativeDirection::Front => player_position.1 -= 1,
            RelativeDirection::Right => player_position.0 += 1,
            RelativeDirection::Left => player_position.0 -= 1,
            RelativeDirection::Back => player_position.1 += 1,
        }
        println!("📍 [POSITION] Joueur {} se déplace vers {:?}", player_id, player_position);
    }
}
