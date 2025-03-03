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
/// use common::message::relativedirection::RelativeDirection;
/// use std::collections::HashMap;
/// use algorithms::radar_view::simulate_movement;
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
/// use algorithms::radar_view::send_action;
/// use common::message::actiondata::ActionData;
/// use common::message::relativedirection::RelativeDirection;
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
/// use algorithms::decrypte::DecodedView;
/// use algorithms::radar_view::choose_accessible_direction;
/// use common::message::relativedirection::RelativeDirection;
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
/// use std::collections::HashMap;
/// use algorithms::radar_view::find_path_to_exit;
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



#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::decrypte::decode_and_format;

    #[test]
    fn test_simulate_movement() {
        let mut position_tracker = HashMap::new();
        position_tracker.insert(1, (5, 5));

        // Tester déplacement vers l'avant (y - 1)
        assert_eq!(simulate_movement(1, RelativeDirection::Front, &position_tracker), Some((5, 4)));

        // Tester déplacement vers l'arrière (y + 1)
        assert_eq!(simulate_movement(1, RelativeDirection::Back, &position_tracker), Some((5, 6)));

        // Tester déplacement vers la droite (x + 1)
        assert_eq!(simulate_movement(1, RelativeDirection::Right, &position_tracker), Some((6, 5)));

        // Tester déplacement vers la gauche (x - 1)
        assert_eq!(simulate_movement(1, RelativeDirection::Left, &position_tracker), Some((4, 5)));

        // Tester un joueur qui n'existe pas
        assert_eq!(simulate_movement(2, RelativeDirection::Front, &position_tracker), None);
    }
    #[test]
    fn test_detect_near_border() {
        let grid_size = (10, 10); // Grille de 10x10

        // Cas où le joueur est sur les bords
        assert_eq!(detect_near_border((0, 5), grid_size), vec![RelativeDirection::Right]); // Bord gauche
        assert_eq!(detect_near_border((9, 5), grid_size), vec![RelativeDirection::Left]); // Bord droit
        assert_eq!(detect_near_border((5, 0), grid_size), vec![RelativeDirection::Back]); // Bord haut
        assert_eq!(detect_near_border((5, 9), grid_size), vec![RelativeDirection::Front]); // Bord bas

        // Cas où le joueur est dans un coin
        assert_eq!(detect_near_border((0, 0), grid_size), vec![RelativeDirection::Right, RelativeDirection::Back]); // Coin haut gauche
        assert_eq!(detect_near_border((9, 0), grid_size), vec![RelativeDirection::Left, RelativeDirection::Back]); // Coin haut droit
        assert_eq!(detect_near_border((0, 9), grid_size), vec![RelativeDirection::Right, RelativeDirection::Front]); // Coin bas gauche
        assert_eq!(detect_near_border((9, 9), grid_size), vec![RelativeDirection::Left, RelativeDirection::Front]); // Coin bas droit

        // Cas où le joueur est au centre de la grille (aucun bord proche)
        assert_eq!(detect_near_border((5, 5), grid_size), vec![]); // Pas de bord
    }
    #[test]
    fn test_compute_absolute_position() {
        let base_pos = (5, 5); // Position de départ

        // Vérifier toutes les positions
        assert_eq!(compute_absolute_position(base_pos, 0), (4, 4)); // Haut gauche
        assert_eq!(compute_absolute_position(base_pos, 1), (5, 4)); // Haut
        assert_eq!(compute_absolute_position(base_pos, 2), (6, 4)); // Haut droite
        assert_eq!(compute_absolute_position(base_pos, 3), (4, 5)); // Gauche
        assert_eq!(compute_absolute_position(base_pos, 4), (5, 5)); // Centre (doit rester inchangé)
        assert_eq!(compute_absolute_position(base_pos, 5), (6, 5)); // Droite
        assert_eq!(compute_absolute_position(base_pos, 6), (4, 6)); // Bas gauche
        assert_eq!(compute_absolute_position(base_pos, 7), (5, 6)); // Bas
        assert_eq!(compute_absolute_position(base_pos, 8), (6, 6)); // Bas droite

        // Cas invalide (cell_index >= 9)
        assert_eq!(compute_absolute_position(base_pos, 9), base_pos); // Doit retourner la position initiale
        assert_eq!(compute_absolute_position(base_pos, 100), base_pos); // Doit retourner la position initiale
    }

    #[test]
    fn test_find_path_to_exit() {
        let mut position_tracker = HashMap::new();
        position_tracker.insert(1, (5, 5)); // Joueur en (5,5)

        let exit_position = (7, 5); // Sortie à droite
        assert_eq!(find_path_to_exit(1, &position_tracker, exit_position), Some(RelativeDirection::Right));

        let exit_position = (3, 5); // Sortie à gauche
        assert_eq!(find_path_to_exit(1, &position_tracker, exit_position), Some(RelativeDirection::Left));

        let exit_position = (5, 7); // Sortie en bas
        assert_eq!(find_path_to_exit(1, &position_tracker, exit_position), Some(RelativeDirection::Back));

        let exit_position = (5, 3); // Sortie en haut
        assert_eq!(find_path_to_exit(1, &position_tracker, exit_position), Some(RelativeDirection::Front));

        let exit_position = (5, 5); // Déjà à la sortie
        assert_eq!(find_path_to_exit(1, &position_tracker, exit_position), Some(RelativeDirection::Front)); // Peut être n'importe quelle direction

        // Tester un joueur qui n'est pas dans `position_tracker`
        assert_eq!(find_path_to_exit(2, &position_tracker, (10, 10)), None);
    }

    #[test]
    fn test_update_player_position() {
        let mut player_position = (5, 5); // Position de départ
        let mut tracker = ExplorationTracker::new();

        // Tester déplacement vers l'avant (Front)
        update_player_position(1, &mut player_position, &ActionData::MoveTo(RelativeDirection::Front), &mut tracker);
        assert_eq!(player_position, (5, 4));
        assert_eq!(tracker.visited_positions.get(&(5, 4)), Some(&1));

        // Tester déplacement vers l'arrière (Back)
        update_player_position(1, &mut player_position, &ActionData::MoveTo(RelativeDirection::Back), &mut tracker);
        assert_eq!(player_position, (5, 5));
        assert_eq!(tracker.visited_positions.get(&(5, 5)), Some(&1));

        // Tester déplacement vers la droite (Right)
        update_player_position(1, &mut player_position, &ActionData::MoveTo(RelativeDirection::Right), &mut tracker);
        assert_eq!(player_position, (6, 5));
        assert_eq!(tracker.visited_positions.get(&(6, 5)), Some(&1));

        // Tester déplacement vers la gauche (Left)
        update_player_position(1, &mut player_position, &ActionData::MoveTo(RelativeDirection::Left), &mut tracker);
        assert_eq!(player_position, (5, 5));
        assert_eq!(tracker.visited_positions.get(&(5, 5)), Some(&2)); // Car déjà visité avant

        // Tester que `tracker` enregistre bien chaque position visitée
        assert!(tracker.visited_positions.contains_key(&(5, 4))); // La position (5,4) doit être enregistrée
        assert!(tracker.visited_positions.contains_key(&(6, 5))); // La position (6,5) aussi
    }

    #[test]
    fn test_decide_action_with_real_radar() {

        let input = "ieysGjGO8papd/a";
        let radar = decode_and_format(input).expect("Erreur de décodage du radar");


        let right_open = is_passage_open(radar.get_vertical_passage(1)  , 2);
        let front_open = is_passage_open(radar.get_horizontal_passage(1)  , 2);
        let left_open = is_passage_open(radar.get_vertical_passage(1)  , 1);

        let action = decide_action(&radar);
        assert_eq!(action, ActionData::MoveTo(RelativeDirection::Front));
    }

    #[test]
    fn test_follow_leader_direction() {
         let input = "ieysGjGO8papd/a";
        let radar = decode_and_format(input).expect("Erreur de décodage du radar");

        println!("\n📝 [RÉSULTAT DÉCODAGE] 📝");
        println!("{:?}", radar);

         if let Some(direction) = follow_leader_direction(&radar, RelativeDirection::Front) {
            println!("👣 Leader direction Front -> Suit : {:?}", direction);
            assert!(matches!(direction, RelativeDirection::Front | RelativeDirection::Right | RelativeDirection::Left | RelativeDirection::Back));
        } else {
            println!("🚧 Aucune direction suivie pour Front");
        }

         if let Some(direction) = follow_leader_direction(&radar, RelativeDirection::Right) {
            println!("👣 Leader direction Right -> Suit : {:?}", direction);
            assert!(matches!(direction, RelativeDirection::Right | RelativeDirection::Front | RelativeDirection::Back | RelativeDirection::Left));
        } else {
            println!("🚧 Aucune direction suivie pour Right");
        }

         if let Some(direction) = follow_leader_direction(&radar, RelativeDirection::Left) {
            println!("👣 Leader direction Left -> Suit : {:?}", direction);
            assert!(matches!(direction, RelativeDirection::Left | RelativeDirection::Front | RelativeDirection::Back | RelativeDirection::Right));
        } else {
            println!("🚧 Aucune direction suivie pour Left");
        }

         if let Some(direction) = follow_leader_direction(&radar, RelativeDirection::Back) {
            println!("👣 Leader direction Back -> Suit : {:?}", direction);
            assert!(matches!(direction, RelativeDirection::Back | RelativeDirection::Left | RelativeDirection::Right | RelativeDirection::Front));
        } else {
            println!("🚧 Aucune direction suivie pour Back");
        }
    }

    #[test]
    fn test_choose_accessible_direction() {
        let input = "ieysGjGO8papd/a";
        let radar = decode_and_format(input).expect("Erreur de décodage du radar");

        println!("\n📝 [RÉSULTAT DÉCODAGE] 📝");
        println!("{:?}", radar);

        let direction = choose_accessible_direction(&radar, vec![
            RelativeDirection::Front,
            RelativeDirection::Right,
            RelativeDirection::Left,
            RelativeDirection::Back,
        ]);

        if let Some(dir) = direction {
            println!("🔄 Direction choisie : {:?}", dir);
            assert!(matches!(dir, RelativeDirection::Front | RelativeDirection::Right | RelativeDirection::Left | RelativeDirection::Back));
        } else {
            println!("⚠️ Aucune direction accessible !");
            assert_eq!(direction, None);
        }

        let direction = choose_accessible_direction(&radar, vec![
            RelativeDirection::Right,
            RelativeDirection::Front,
            RelativeDirection::Back,
            RelativeDirection::Left,
        ]);

        if let Some(dir) = direction {
            println!("🔄 Direction choisie : {:?}", dir);
            assert!(matches!(dir, RelativeDirection::Right | RelativeDirection::Front | RelativeDirection::Back | RelativeDirection::Left));
        } else {
            println!("⚠️ Aucune direction accessible !");
            assert_eq!(direction, None);
        }

        let direction = choose_accessible_direction(&radar, vec![
            RelativeDirection::Back,
        ]);

        println!("🚧 Test avec Back bloqué : {:?}", direction);
        assert_eq!(direction, None);
    }

    #[test]
    fn test_follower_choose_action() {
        let input = "ieysGjGO8papd/a";
        let radar_data = decode_and_format(input).expect("Erreur de décodage du radar");

        println!("\n📝 [RÉSULTAT DÉCODAGE] 📝");
        println!("{:?}", radar_data);

        let player_id = 2;

        let shared_leader_action = Arc::new(Mutex::new(Some(ActionData::MoveTo(RelativeDirection::Right))));
        let action = follower_choose_action(player_id, &radar_data, &shared_leader_action);
        println!("👣 Cas 1: Follower suit -> {:?}", action);
        assert!(matches!(action, ActionData::MoveTo(RelativeDirection::Right) | ActionData::MoveTo(_)));

        let shared_leader_action = Arc::new(Mutex::new(Some(ActionData::MoveTo(RelativeDirection::Front))));
        let action = follower_choose_action(player_id, &radar_data, &shared_leader_action);
        println!("🚧 Cas 2: Leader Front bloqué -> Follower s'adapte: {:?}", action);
        assert!(matches!(action, ActionData::MoveTo(_))); // Doit trouver une autre direction

        let shared_leader_action = Arc::new(Mutex::new(None));
        let action = follower_choose_action(player_id, &radar_data, &shared_leader_action);
        println!("🔄 Cas 3: Aucun leader -> Follower décide seul: {:?}", action);
        assert!(matches!(action, ActionData::MoveTo(_))); // Doit prendre une décision seul

    }


    #[test]
    fn test_choose_least_visited_direction() {
         let input = "ieysGjGO8papd/a";
        let radar_data = decode_and_format(input).expect("Erreur de décodage du radar");

        println!("\n📝 [RÉSULTAT DÉCODAGE] 📝");
        println!("{:?}", radar_data);

        let player_id = 1;

         let mut position_tracker = HashMap::new();
        position_tracker.insert(player_id, (5, 5));

         let mut tracker = ExplorationTracker::new();
        tracker.visited_positions.insert((5, 4), 3); // Front : 3 visites
        tracker.visited_positions.insert((6, 5), 5); // Right : 5 visites
        tracker.visited_positions.insert((4, 5), 2); // Left : 2 visites
        tracker.visited_positions.insert((5, 6), 1); // Back : 1 visite

         let action = choose_least_visited_direction(player_id, &radar_data, &mut tracker, &position_tracker);
        println!("🔄 Direction choisie : {:?}", action);
        assert_eq!(action, ActionData::MoveTo(RelativeDirection::Back));

         tracker.visited_positions.insert((5, 4), 3);
        tracker.visited_positions.insert((6, 5), 3);
        tracker.visited_positions.insert((4, 5), 3);
        tracker.visited_positions.insert((5, 6), 3);

        let action = choose_least_visited_direction(player_id, &radar_data, &mut tracker, &position_tracker);
        println!("🔄 Cas où toutes les directions sont visitées également : {:?}", action);
        assert!(matches!(action, ActionData::MoveTo(_))); // Peut être n'importe quelle direction

         tracker.visited_positions.insert((5, 4), 9999);
        tracker.visited_positions.insert((6, 5), 9999);
        tracker.visited_positions.insert((4, 5), 9999);
        tracker.visited_positions.insert((5, 6), 9999);

        let action = choose_least_visited_direction(player_id, &radar_data, &mut tracker, &position_tracker);
        println!("🚧 Cas où toutes les directions sont trop visitées : {:?}", action);
        assert!(matches!(action, ActionData::MoveTo(_))); // `decide_action` prendra le relais
    }

    #[test]
    fn test_leader_choose_action() {
        let input = "ieysGjGO8papd/a";
        let radar_data = decode_and_format(input).expect("Erreur de décodage du radar");

        println!("\n📝 [RÉSULTAT DÉCODAGE] 📝");
        println!("{:?}", radar_data);

        let player_id = 1;

        let mut position_tracker = HashMap::new();
        position_tracker.insert(player_id, (5, 5));

        let mut tracker = ExplorationTracker::new();
        tracker.visited_positions.insert((5, 4), 3);
        tracker.visited_positions.insert((6, 5), 1);
        tracker.visited_positions.insert((4, 5), 5);
        tracker.visited_positions.insert((5, 6), 2);

        let exit_position = Arc::new(Mutex::new(Some((7, 5))));

        let action = leader_choose_action(player_id, &radar_data, None, None, &mut tracker, &position_tracker, &exit_position);
        println!("🚀 Cas 1: Le leader connaît la sortie -> {:?}", action);
        assert_eq!(action, ActionData::MoveTo(RelativeDirection::Right));

        let exit_position = Arc::new(Mutex::new(None));
        let action = leader_choose_action(player_id, &radar_data, Some((10, 10)), None, &mut tracker, &position_tracker, &exit_position);
        println!("🗺️ Cas 2: Pas de sortie connue, stratégie basée sur la taille -> {:?}", action);
        assert!(matches!(action, ActionData::MoveTo(_)));


        let action = leader_choose_action(player_id, &radar_data, None, Some(90.0), &mut tracker, &position_tracker, &exit_position);
        println!("🧭 Cas 3: Le leader suit la boussole -> {:?}", action);
        assert!(matches!(action, ActionData::MoveTo(_)));


        let action = leader_choose_action(player_id, &radar_data, None, None, &mut tracker, &position_tracker, &exit_position);
        println!("🔄 Cas 4: Aucune info dispo -> Direction la moins visitée -> {:?}", action);
        assert_eq!(action, ActionData::MoveTo(RelativeDirection::Front));
    }
}
