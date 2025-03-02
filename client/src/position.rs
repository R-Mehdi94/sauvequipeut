use common::message::relativedirection::RelativeDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn update(&mut self, direction: &RelativeDirection) {
        match direction {
            RelativeDirection::Front => self.y -= 1,
            RelativeDirection::Back => self.y += 1,
            RelativeDirection::Left => self.x -= 1,
            RelativeDirection::Right => self.x += 1,
        }
    }
}

