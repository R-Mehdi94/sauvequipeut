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
}

impl PlayerMemory {
    pub fn new(i: i32) -> Self {
        Self {
            history: VecDeque::with_capacity(HISTORY_SIZE),
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
    tracker: &ExplorationTracker,
    position_tracker: &HashMap<u32, (i32, i32)>,
    player_memory: &mut PlayerMemory,
) -> ActionData {
    let current_position = *position_tracker.get(&player_id).unwrap();
    let all_directions = vec![
        RelativeDirection::Front,
        RelativeDirection::Right,
        RelativeDirection::Left,
        RelativeDirection::Back,
    ];

    // 🔍 1. Filtrer uniquement les directions accessibles
    let accessible_directions: Vec<RelativeDirection> = all_directions
        .iter()
        .filter(|&&dir| choose_accessible_direction(radar_data, vec![dir]).is_some())
        .cloned()
        .collect();

    if accessible_directions.is_empty() {
        println!("🚨 [ERREUR] Aucune direction accessible ! Forcer une stratégie aléatoire.");
        return decide_action(radar_data); // Prend une décision par défaut si bloqué
    }

    // 🔍 2. Choisir la direction la moins visitée parmi les accessibles
    let mut best_direction = None;
    let mut lowest_visits = usize::MAX;

    for &direction in &accessible_directions {
        if let Some(new_position) = simulate_movement(player_id, direction, position_tracker) {
            let visit_count = tracker.visited_positions.get(&new_position).cloned().unwrap_or(0);

            // 🔥 Priorité aux moins visitées ET qui ne sont pas dans la mémoire récente du joueur
            if visit_count < lowest_visits && !player_memory.history.contains(&new_position) {
                lowest_visits = visit_count;
                best_direction = Some(direction);
            }
        }
    }

    if let Some(direction) = best_direction {
        if let Some(new_position) = simulate_movement(player_id, direction, position_tracker) {
            player_memory.update_position(new_position);
        }
        println!(
            "✅ [DIRECTION] Joueur {} choisit {:?} avec {} visites",
            player_id, direction, lowest_visits
        );
        return ActionData::MoveTo(direction);
    }

    // 🚨 Si toutes les options sont en boucle → Prendre une direction aléatoire pour casser la boucle
    if player_memory.is_looping() {
        println!("🔄 [ALERTE] Joueur {} détecte une boucle ! Forcer une nouvelle direction.", player_id);
        return decide_action(radar_data); // Prend une décision par défaut si boucle
    }

    println!("⚠️ [DIRECTION] Aucune direction optimale trouvée, retour à la stratégie plombier.");
    decide_action(radar_data)
}



