use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::keycode::KeyInput;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method", content = "params")]
pub enum ToFrontend {
    Open {
        id: Uuid,
        title: String,
        text: String,
    },
    UpdateText {
        id: Uuid,
        text: String,
    },
    SetCursorPosition {
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method", content = "params")]
pub enum ToBackend {
    KeyPressed(KeyInput),
    MouseInput { line: usize, column: usize },
}
