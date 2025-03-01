use std::collections::{HashMap, VecDeque};
use rand::prelude::IndexedRandom;
use rand::thread_rng;
use common::message::actiondata::ActionData;
use common::message::relativedirection::RelativeDirection;
use crate::decrypte::DecodedView;
use crate::exploration_tracker::ExplorationTracker;
use crate::radar_view::{choose_accessible_direction, decide_action, simulate_movement};

const HISTORY_SIZE: usize = 5;

pub struct PlayerMemory {
    pub(crate) history: VecDeque<(i32, i32)>,

    pub last_direction: Option<RelativeDirection>,
}

impl PlayerMemory {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(HISTORY_SIZE),
            last_direction: None,
        }
    }

    pub fn update_position(&mut self, new_position: (i32, i32)) {
        if self.history.len() == HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(new_position);
    }

    pub fn is_looping(&self) -> bool {
        let unique_positions: std::collections::HashSet<_> = self.history.iter().collect();
        unique_positions.len() < self.history.len()
    }
}




pub fn choose_least_visited_direction(
    player_id: u32,
    radar_data: &DecodedView,
    tracker: &mut ExplorationTracker, // 📌 Tracker mutable pour enregistrer la nouvelle position
    position_tracker: &HashMap<u32, (i32, i32)>,
) -> ActionData {
    let current_position = *position_tracker.get(&player_id).unwrap();
    let all_directions = vec![
        RelativeDirection::Front,
        RelativeDirection::Right,
        RelativeDirection::Left,
        RelativeDirection::Back,
    ];

    // 🔍 1. Récupérer les directions accessibles
    let accessible_directions: Vec<RelativeDirection> = all_directions
        .iter()
        .filter(|&&dir| choose_accessible_direction(radar_data, vec![dir]).is_some())
        .cloned()
        .collect();

    if accessible_directions.is_empty() {
        println!("🚨 [ERREUR] Aucune direction accessible ! Forcer une stratégie aléatoire.");
        return decide_action(radar_data);
    }

    // 🔍 2. Vérifier si une boucle est détectée
    if tracker.is_recently_visited(current_position) {
        println!("🔄 [DÉTECTION DE BOUCLE] Joueur {} évite la dernière direction prise.", player_id);

        let alternative_directions: Vec<_> = accessible_directions
            .iter()
            .filter(|&&dir| !tracker.is_recently_visited(simulate_movement(player_id, dir, position_tracker).unwrap_or(current_position)))
            .cloned()
            .collect();

        if let Some(new_direction) = alternative_directions.choose(&mut rand::thread_rng()) {
            println!("🛑 [NOUVELLE STRATÉGIE] Direction différente choisie : {:?}", new_direction);
            tracker.mark_position(current_position,*new_direction);
            return ActionData::MoveTo(*new_direction);
        }

        println!("⚠️ [AUCUNE AUTRE OPTION] Forçage d’une direction aléatoire.");
    }

    // 🔍 3. Choisir la direction la moins visitée
    let mut best_direction = None;
    let mut lowest_visits = usize::MAX;

    for &direction in &accessible_directions {
        if let Some(new_position) = simulate_movement(player_id, direction, position_tracker) {
            let visit_count = tracker.visited_positions.get(&new_position).cloned().unwrap_or(0);

            // 🔥 Priorité aux moins visitées
            if visit_count < lowest_visits {
                lowest_visits = visit_count;
                best_direction = Some(direction);
            }
        }
    }

    if let Some(direction) = best_direction {
        tracker.mark_position(current_position,direction);
        println!(
            "✅ [DIRECTION] Joueur {} choisit {:?} avec {} visites",
            player_id, direction, lowest_visits
        );
        return ActionData::MoveTo(direction);
    }

    // 🔍 4. Dernier recours : choisir aléatoirement si aucune meilleure option n’est trouvée
    println!("⚠️ [DIRECTION] Aucune direction optimale trouvée, retour à la stratégie aléatoire.");
    let chosen_direction = *accessible_directions.choose(&mut rand::thread_rng()).unwrap();
    tracker.mark_position(current_position,chosen_direction);
    ActionData::MoveTo(chosen_direction)
}
