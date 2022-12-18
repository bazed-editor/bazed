use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::keycode::KeyInput;

/// Id of a request, which is any RPC invocation that expects a response.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestId(pub Uuid);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CaretPosition {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method", content = "params")]
pub enum ToFrontend {
    OpenDocument {
        document_id: Uuid,
        path: Option<PathBuf>,
        text: String,
    },
    UpdateDocument {
        document_id: Uuid,
        text: String,
        carets: Vec<CaretPosition>,
    },

    ViewOpenedResponse {
        request_id: RequestId,
        view_id: Uuid,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method", content = "params")]
pub enum ToBackend {
    KeyPressed {
        view_id: Uuid,
        input: KeyInput,
    },

    /// Mouse was clicked. The coordinates are absolute.
    MouseInput {
        view_id: Uuid,
        position: CaretPosition,
    },

    ViewOpened {
        request_id: RequestId,
        document_id: Uuid,
        height: usize,
        width: usize,
    },
}
