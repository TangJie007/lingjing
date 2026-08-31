use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EngineState {
    pub media_id: Option<String>,
    pub title: Option<String>,
    pub media_type: Option<String>,
    pub uri: Option<String>,
    pub playing: bool,
    pub volume: f64,
    pub muted: bool,
    pub current_time: f64,
    pub duration: f64,
    pub error: Option<String>,
    #[serde(default)]
    pub user_paused: bool,
    #[serde(default)]
    pub low_power: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MonitorTile {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetWallpaperPayload {
    pub id: String,
    pub title: String,
    pub media_type: String,
    pub uri: String,
}

pub struct EngineHandle {
    pub state: Mutex<EngineState>,
}

impl Default for EngineHandle {
    fn default() -> Self {
        Self {
            state: Mutex::new(EngineState {
                playing: true,
                volume: 0.8,
                muted: false,
                ..Default::default()
            }),
        }
    }
}
