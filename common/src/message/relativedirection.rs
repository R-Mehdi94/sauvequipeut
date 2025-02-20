use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum RelativeDirection {
    Left,
    Right,
    Front,
    Back,
}
