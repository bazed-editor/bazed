//! User-level operation that can be performed on a [Buffer].
//! This includes edit and movement operations.
//! These will occur at the caret positions, and are thus only used for directly userfacing operations

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
    Edit(EditOp),
    Movement(MovementOp),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum DocumentOp {
    Save,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum EditOp {
    Insert(String),
    Backspace,
    Undo,
    Redo,
}

// currently just used in tests
#[allow(unused)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum MovementOp {
    Left,
    Right,
    Up,
    Down,
    StartOfLine,
    EndOfLine,
    TopOfViewport,
    BottomOfViewport,
}
