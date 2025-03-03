use std::collections::HashMap;
use common::message::relativedirection::RelativeDirection;

pub struct ExplorationTracker {
    pub(crate) visited_positions: HashMap<(i32, i32), usize>,
    last_positions: Vec<(i32, i32)>,

    last_direction: Option<RelativeDirection>,

}

impl ExplorationTracker {
    pub fn new() -> Self {
        Self {
            visited_positions: HashMap::new(),
            last_positions: Vec::new(),
            last_direction: None,
        }
    }
    pub fn mark_position(&mut self, position: (i32, i32), direction: RelativeDirection) {
        let count = self.visited_positions.entry(position).or_insert(0);
        *count += 1;

        println!("📝 [DEBUG] Ajout de la position {:?} avec direction {:?}", position, direction);
        println!("📌 [DEBUG] Avant ajout, last_positions: {:?}", self.last_positions);

        self.last_positions.push(position);
        if self.last_positions.len() > 8 {
            self.last_positions.remove(0);
        }

        self.last_direction = Some(direction);

        if self.last_positions.len() >= 3 && self.last_positions[..self.last_positions.len() - 1].contains(&position) {
            println!("🔄 [ALERTE] Boucle détectée à {:?}", position);
        }

        println!("📌 [DEBUG] Après ajout, last_positions: {:?}", self.last_positions);
        println!("➡️ [DEBUG] Dernière direction prise : {:?}", self.last_direction);
    }

    pub fn is_recently_visited(&self, position: (i32, i32)) -> bool {
        if self.last_positions.len() < 5 {
            return false;
        }
        println!("🔍 [DEBUG] Vérification de la position {} dans is_recently_visited() {}", position.0, position.1);
        self.last_positions[..self.last_positions.len() - 1].contains(&position)
    }


}