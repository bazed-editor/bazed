use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::keycode::KeyInput;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CaretPosition {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method", content = "params")]
pub enum ToFrontend {
    Open {
        id: Uuid,
        title: String,
        text: String,
    },
    UpdateDocument {
        id: Uuid,
        text: String,
        carets: Vec<CaretPosition>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method", content = "params")]
pub enum ToBackend {
    KeyPressed(KeyInput),
    MouseInput(CaretPosition),
}
