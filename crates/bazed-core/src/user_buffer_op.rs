//! User-level operation that can be performed on a [Buffer].
//! This includes edit and movement operations.
//! These will occur at the caret positions, and are thus only used for directly userfacing operations

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum BufferOp {
    Edit(EditOp),
    Movement(MovementOp),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum EditOp {
    Insert(String),
    Backspace,
    Undo,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum MovementOp {
    Left,
    Right,
    Up,
    Down,
}
