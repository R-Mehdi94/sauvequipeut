use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RadarView {
    RadarView(String),
}
