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
use crate::decrypte::{is_passage_open, DecodedView, RadarCell};
use crate::exploration_tracker::ExplorationTracker;
use crate::hint::{direction_from_angle, direction_from_grid_size};


/// Sélectionne la direction la **moins visitée** par le joueur pour explorer la carte.
///
/// # Paramètres
/// - `player_id`: L'identifiant du joueur.
/// - `radar_data`: Les données du radar pour détecter les passages accessibles.
/// - `tracker`: L'objet qui suit les positions visitées.
/// - `position_tracker`: La carte des positions actuelles des joueurs.
///
/// # Retourne
/// - Une action `MoveTo` vers la direction la moins explorée.
///
/// # Exemple
/// ```
/// use ma_lib::choose_least_visited_direction;
/// use common::message::actiondata::ActionData;
/// use common::message::relativedirection::RelativeDirection;
/// ````
pub fn choose_least_visited_direction(
    player_id: u32,
    radar_data: &DecodedView,
    tracker: &mut ExplorationTracker,
    position_tracker: &HashMap<u32, (i32, i32)>,
) -> ActionData {

    let mut best_direction = None;
    let mut lowest_visits = usize::MAX;

    for direction in &[RelativeDirection::Front, RelativeDirection::Right, RelativeDirection::Left, RelativeDirection::Back] {
        if let Some(new_position) = simulate_movement(player_id, *direction, position_tracker) {
            let visit_count = tracker.visited_positions.get(&new_position).cloned().unwrap_or(0);
            if visit_count < lowest_visits {
                lowest_visits = visit_count;
                best_direction = Some(*direction);
            }
        }
    }

    if let Some(direction) = best_direction {
        return ActionData::MoveTo(direction);
    }

    println!("⚠️ [DIRECTION] Aucune direction optimale trouvée, activation de la stratégie plombier.");
    decide_action(radar_data)
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

/// Simule un mouvement dans une direction pour estimer la **nouvelle position** du joueur.
///
/// # Paramètres
/// - `player_id`: L'identifiant du joueur.
/// - `direction`: La direction envisagée.
/// - `position_tracker`: La carte des positions des joueurs.
///
/// # Retourne
/// - `Option<(i32, i32)>` avec les nouvelles coordonnées du joueur si elles sont valides.
///
/// # Exemple
/// ```
/// use ma_lib::simulate_movement;
/// use common::message::relativedirection::RelativeDirection;
/// use std::collections::HashMap;
///
/// let mut position_tracker = HashMap::new();
/// position_tracker.insert(1, (5, 5));
///
/// let new_position = simulate_movement(1, RelativeDirection::Front, &position_tracker);
/// assert_eq!(new_position, Some((5, 4)));
/// ```
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

//////////////////////////////////////////////////////////////////////////////////////////////////////


pub fn leader_choose_action(
    player_id: u32,
    radar_data: &DecodedView,
    grid_size: Option<(u32, u32)>,
    compass_angle: Option<f32>,
    tracker: &mut ExplorationTracker,
    position_tracker: &HashMap<u32, (i32, i32)>,
    exit_position: &Arc<Mutex<Option<(i32, i32)>>>,
) -> ActionData {
    let current_position = *position_tracker.get(&player_id).unwrap();
    if tracker.is_recently_visited(current_position) {
        println!("🔄 [ALERTE] Joueur {} est coincé dans une boucle ! Recherche d'un nouveau chemin...", player_id);

        choose_least_visited_direction(player_id, radar_data, tracker, position_tracker);

    }
    //  **Vérifier si on connaît la sortie**
    if let Some(exit_pos) = *exit_position.lock().unwrap() {
        println!("🚪 [INFO] Joueur {} sait où est la sortie en {:?}", player_id, exit_pos);
        if let Some(direction) = find_path_to_exit(player_id, position_tracker, exit_pos) {
            println!("🚀 [SORTIE] Joueur {} se dirige vers {:?}", player_id, direction);
            return ActionData::MoveTo(direction);
        }
    }

    if let Some(grid) = grid_size {
        let near_borders = detect_near_border(current_position, grid);
        if !near_borders.is_empty() {
            println!("🏁 [INFO] Joueur {} proche d'un bord. Stratégie ajustée.", player_id);
        }
    }

    // 🗺️ **Exploration basée sur la taille du labyrinthe**
    if let Some((cols, rows)) = grid_size {
        println!("🗺️ [LEADER] Taille labyrinthe : {} colonnes x {} lignes.", cols, rows);
        let direction_priority = direction_from_grid_size(grid_size);

        if let Some(direction) = choose_accessible_direction(radar_data, direction_priority) {
            if let Some(new_position) = simulate_movement(player_id, direction, position_tracker) {
                let visit_count = tracker.visited_positions.get(&new_position).cloned().unwrap_or(0);

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

    // 🧭 **Si on a la boussole, on essaye de l'utiliser**
    if let Some(angle) = compass_angle {
        println!("🧭 [LEADER] Utilisation de la boussole : {:.2}°", angle);
        let direction_priority = direction_from_angle(angle);

        if let Some(direction) = choose_accessible_direction(radar_data, direction_priority) {
            if let Some(new_position) = simulate_movement(player_id, direction, position_tracker) {
                let visit_count = tracker.visited_positions.get(&new_position).cloned().unwrap_or(0);

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

    // 🚀 **Dernier recours : Prendre la direction la moins visitée**
    println!("🔄 [INFO] Aucun choix évident, recours à la direction la moins visitée.");
    //choose_least_visited_direction(player_id, radar_data, tracker, position_tracker)
    decide_action(radar_data)

}

//////////////////////////////////////////////////////////////////////////////////////////////////////
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

//////////////////////////////////////////////////////////////////////////////////////////////////////

/// Envoie une action au serveur et assure la **gestion des erreurs en cas de connexion interrompue**.
///
/// # Paramètres
/// - `player_id`: L'identifiant du joueur.
/// - `action`: L'action à envoyer.
/// - `tx`: Un canal pour envoyer l'action localement.
/// - `stream`: La connexion `TcpStream` vers le serveur.
///
/// # Exemple
/// ```no_run
/// use std::net::TcpStream;
/// use std::sync::mpsc::channel;
/// use ma_lib::{send_action, PlayerAction};
/// use common::message::actiondata::ActionData;
///
/// let (tx, _rx) = channel();
/// let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
/// send_action(1, ActionData::MoveTo(RelativeDirection::Front), &tx, &mut stream);
/// ```
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

    if let Err(_e) = send_message(stream, &Message::Action(action)) {
        warn!("🔄 Tentative de reconnexion dans 2 secondes...");
        thread::sleep(Duration::from_secs(2));
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

/// Vérifie quelles **directions accessibles** sont disponibles à partir des données radar.
///
/// # Paramètres
/// - `radar`: Vue radar actuelle du joueur.
/// - `directions`: Liste des directions prioritaires.
///
/// # Retourne
/// - `Some(RelativeDirection)` si une direction accessible est trouvée.
/// - `None` si aucune direction n'est ouverte.
///
/// # Exemple
/// ```
/// use ma_lib::choose_accessible_direction;
/// use common::message::relativedirection::RelativeDirection;
/// use crate::decrypte::DecodedView;
///
/// let radar = DecodedView::default();
/// let direction = choose_accessible_direction(&radar, vec![RelativeDirection::Front]);
/// ```
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

//////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn follow_leader_direction(radar: &DecodedView, leader_direction: RelativeDirection) -> Option<RelativeDirection> {
    let direction_priority = match leader_direction {
        RelativeDirection::Front => vec![RelativeDirection::Front, RelativeDirection::Right, RelativeDirection::Left, RelativeDirection::Back],
        RelativeDirection::Right => vec![RelativeDirection::Right, RelativeDirection::Front, RelativeDirection::Back, RelativeDirection::Left],
        RelativeDirection::Left => vec![RelativeDirection::Left, RelativeDirection::Front, RelativeDirection::Back, RelativeDirection::Right],
        RelativeDirection::Back => vec![RelativeDirection::Back, RelativeDirection::Left, RelativeDirection::Right, RelativeDirection::Front],
    };

    choose_accessible_direction(radar, direction_priority)
}

//////////////////////////////////////////////////////////////////////////////////////////////////////
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

//////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn update_player_position(
    player_id: u32,
    player_position: &mut (i32, i32),
    action: &ActionData,
    tracker: &mut ExplorationTracker,
) {
    if let ActionData::MoveTo(direction) = action {
        match direction {
            RelativeDirection::Front => player_position.1 -= 1,
            RelativeDirection::Right => player_position.0 += 1,
            RelativeDirection::Left => player_position.0 -= 1,
            RelativeDirection::Back => player_position.1 += 1,
        }
        println!("📍 [POSITION] Joueur {} se déplace vers {:?}", player_id, player_position);
        tracker.mark_position(*player_position,*direction);

    }

}

//////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn compute_absolute_position(current_pos: (i32, i32), cell_index: usize) -> (i32, i32) {
    match cell_index {
        0 => (current_pos.0 - 1, current_pos.1 - 1), // Haut gauche
        1 => (current_pos.0, current_pos.1 - 1),     // Haut
        2 => (current_pos.0 + 1, current_pos.1 - 1), // Haut droite
        3 => (current_pos.0 - 1, current_pos.1),     // Gauche
        4 => (current_pos.0, current_pos.1),         // Centre (position actuelle)
        5 => (current_pos.0 + 1, current_pos.1),     // Droite
        6 => (current_pos.0 - 1, current_pos.1 + 1), // Bas gauche
        7 => (current_pos.0, current_pos.1 + 1),     // Bas
        8 => (current_pos.0 + 1, current_pos.1 + 1), // Bas droite
        _ => current_pos,
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn detect_near_border(
    position: (i32, i32),
    grid_size: (u32, u32),
) -> Vec<RelativeDirection> {
    let mut directions = vec![];

    let (cols, rows) = grid_size;
    let (x, y) = position;

    if x == 0 {
        println!("🧱 [INFO] Joueur proche du bord gauche !");
        directions.push(RelativeDirection::Right);
    }
    if x == (cols as i32 - 1) {
        println!("🧱 [INFO] Joueur proche du bord droit !");
        directions.push(RelativeDirection::Left);
    }
    if y == 0 {
        println!("🧱 [INFO] Joueur proche du bord supérieur !");
        directions.push(RelativeDirection::Back);
    }
    if y == (rows as i32 - 1) {
        println!("🧱 [INFO] Joueur proche du bord inférieur !");
        directions.push(RelativeDirection::Front);
    }

    directions
}

//////////////////////////////////////////////////////////////////////////////////////////////////////
/// Trouve le **chemin vers la sortie** en fonction de la position actuelle du joueur.
///
/// # Paramètres
/// - `player_id`: Identifiant du joueur.
/// - `position_tracker`: Carte des positions des joueurs.
/// - `exit_position`: Position de la sortie.
///
/// # Retourne
/// - `Some(RelativeDirection)` si une direction vers la sortie est trouvée.
/// - `None` sinon.
///
/// # Exemple
/// ```
/// use ma_lib::find_path_to_exit;
/// use std::collections::HashMap;
/// use common::message::relativedirection::RelativeDirection;
///
/// let mut position_tracker = HashMap::new();
/// position_tracker.insert(1, (5, 5));
///
/// let direction = find_path_to_exit(1, &position_tracker, (7, 5));
/// assert_eq!(direction, Some(RelativeDirection::Right));
/// ```
pub fn find_path_to_exit(
    player_id: u32,
    position_tracker: &HashMap<u32, (i32, i32)>,
    exit_position: (i32, i32)
) -> Option<RelativeDirection> {
    let current_position = position_tracker.get(&player_id)?;

    let dx = exit_position.0 - current_position.0;
    let dy = exit_position.1 - current_position.1;

    if dx.abs() > dy.abs() {
        if dx > 0 {
            Some(RelativeDirection::Right)
        } else {
            Some(RelativeDirection::Left)
        }
    } else {
        if dy > 0 {
            Some(RelativeDirection::Back)
        } else {
            Some(RelativeDirection::Front)
        }
    }
}