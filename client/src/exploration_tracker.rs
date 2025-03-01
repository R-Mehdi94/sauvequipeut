use std::collections::HashMap;
use std::sync::{Arc, Mutex};
struct ExplorationTracker {
    visited_positions: HashMap<(i32, i32), usize>,
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
        if self.last_positions.len() > 3 { // Garde seulement les 3 dernières positions
            self.last_positions.remove(0);
        }
    }

    pub fn is_recently_visited(&self, position: (i32, i32)) -> bool {
        self.last_positions.contains(&position)
    }
}
