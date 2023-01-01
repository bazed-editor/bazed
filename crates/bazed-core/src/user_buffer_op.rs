//! User-level operation that can be performed on a [crate::buffer::Buffer].
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

#[derive(Debug, Clone)]
pub(crate) enum BufferOp {
    Insert(String),
    Delete(Trajectory),
    Undo,
    Redo,
    DeleteSelected,
    Move(Motion),
    /// Expand or change the selection
    Selection(Motion),
    /// Create a new cursor at the location the motion targets
    NewCaret(Motion),
}

/// A motion, either character-wise or defined by some higher-level semantic target.
/// Conceptually similar to motions in vim (`w`, `t$`)
#[allow(unused)]
#[derive(Debug, Clone)]
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
    PrevWordBoundary(WordBoundaryType),
    FindNext(hotsauce::Regex),
    FindPrev(hotsauce::Regex),
}
