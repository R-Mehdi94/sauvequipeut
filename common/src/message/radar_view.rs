use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RadarViewResult {
    radar_view_result: String,
}
