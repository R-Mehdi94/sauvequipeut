use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct ExplorationTracker {
    pub visited_positions: HashMap<(i32, i32), usize>,
}

impl ExplorationTracker {
    pub fn new() -> Self {
        Self {
            visited_positions: HashMap::new(),
        }
    }

    pub fn mark_position(&mut self, position: (i32, i32)) {
        let count = self.visited_positions.entry(position).or_insert(0);
        *count += 1;
    }

    pub fn has_cycle(&self, position: (i32, i32)) -> bool {
        self.visited_positions.get(&position).map_or(false, |&count| count >= 3)
    }

    pub fn reset(&mut self) {
        self.visited_positions.clear();
    }
}
