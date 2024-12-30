use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum HintData {
    RelativeCompass { angle: f32 },
    GridSize { columns: u32, rows: u32 },
    Secret(u64),
    SOSHelper,
}
