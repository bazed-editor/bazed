use std::path::PathBuf;

use bazed_input_mapper::input_event::KeyInput;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Id of a request, which is any RPC invocation that expects a response.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestId(pub Uuid);

/// Absolute position within a document.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Coordinate {
    pub line: usize,
    pub col: usize,
}

/// A region (i.e. a selection, a caret) defined by two absolute coordinates.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CoordinateRegion {
    pub head: Coordinate,
    pub tail: Coordinate,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ViewData {
    pub first_line: usize,
    pub text: Vec<String>,
    /// caret positions are absolute
    pub carets: Vec<CoordinateRegion>,
    pub vim_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method", content = "params")]
pub enum ToFrontend {
    /// Sent when a new view should be opened.
    OpenView {
        view_id: Uuid,
        path: Option<PathBuf>,
        view_data: ViewData,
    },
    /// Sent whenever anything in the view changed, i.e. the content,
    /// the viewport, or a caret position
    UpdateView { view_id: Uuid, view_data: ViewData },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "method", content = "params")]
pub enum ToBackend {
    KeyPressed {
        view_id: Uuid,
        input: KeyInput,
    },
    /// Mouse was clicked notification.
    MouseInput {
        view_id: Uuid,
        /// Absolute coordinates, see [Coordinate]
        position: Coordinate,
    },
    /// Mouse wheel turned notification.
    MouseScroll {
        view_id: Uuid,
        /// Positive or negative values mean scrolling down or up respectively
        line_delta: i32,
    },
    /// Send when the viewport for a given view has changed,
    /// i.e. because the window was resized or the user scrolled.
    ViewportChanged {
        view_id: Uuid,
        height: usize,
    },
}
