use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum RelativeDirection {
    Left,
    Right,
    Front,
    Back,
}
