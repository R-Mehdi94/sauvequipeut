use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RelativeDirection {
    Left,
    Right,
    Forward,
    Backward,
}
