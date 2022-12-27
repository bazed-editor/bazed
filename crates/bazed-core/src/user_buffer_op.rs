//! User-level operation that can be performed on a [Buffer].
//! This includes edit and movement operations.
//! These will occur at the caret positions, and are thus only used for directly userfacing operations

use crate::word_boundary::WordBoundaryType;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Trajectory {
    Forwards,
    Backwards,
}

/// Category of an edit, used for grouping operations into undo-groups
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EditType {
    Insert,
    Delete,
    /// Catch-all type for any operation that shouldn't be grouped at all
    Other,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum Operation {
    Document(DocumentOp),
    Buffer(BufferOp),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum DocumentOp {
    Save,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum BufferOp {
    Insert(String),
    Delete(Trajectory),
    Undo,
    Redo,
    Move(Motion),
    /// Expand or change the selection
    Selection(Motion),
}

/// A motion, either character-wise or defined by some higher-level semantic target.
/// Conceptually similar to motions in vim (`w`, `t$`)
#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum Motion {
    Left,
    Right,
    Up,
    Down,
    StartOfLine,
    EndOfLine,
    TopOfViewport,
    BottomOfViewport,
    NextWordBoundary(WordBoundaryType),
}
