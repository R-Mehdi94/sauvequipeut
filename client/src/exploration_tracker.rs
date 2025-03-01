use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ExplorationTracker {
    pub(crate) visited_positions: HashMap<(i32, i32), usize>,
    last_positions: Vec<(i32, i32)>, // Mémorisation des 3 dernières positions
}



impl ExplorationTracker {
    pub fn new() -> Self {
        Self {
            visited_positions: HashMap::new(),
            last_positions: Vec::new(),
        }
    }
    pub fn mark_position(&mut self, position: (i32, i32)) {
        let count = self.visited_positions.entry(position).or_insert(0);
        *count += 1;

        self.last_positions.push(position);
        if self.last_positions.len() > 8 { // Garde les 5 dernières positions
            self.last_positions.remove(0);
        }

        // 🔄 Détection de boucle : 3 mêmes positions répétées
        if self.last_positions.len() >= 3 && self.last_positions[0] == self.last_positions[2] {
            println!("🔄 [ALERTE] Boucle détectée à {:?}", position);
        }

        // 📌 DEBUG: Affichage complet des dernières positions
        println!("📌 [DEBUG] État de last_positions: {:?}", self.last_positions);
    }


    pub fn is_recently_visited(&self, position: (i32, i32)) -> bool {
        println!("🔍 [DEBUG] Vérification de la position {} dans is_recently_visited() {}", position.0, position.1);
        self.last_positions.contains(&position)
    }

}
