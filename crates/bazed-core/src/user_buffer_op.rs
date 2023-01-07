//! User-level operation that can be performed on a [crate::buffer::Buffer].
//! This includes edit and movement operations.
//! These will occur at the caret positions, and are thus only used for directly userfacing operations

use crate::word_boundary::WordBoundaryType;

/// Category of an edit, used for grouping operations into undo-groups
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EditType {
    Insert,
    Delete,
    Replace,
    /// Catch-all type for any operation that shouldn't be grouped at all
    Other,
}

#[derive(Debug, Clone)]
pub(crate) enum BufferOp<'a> {
    Insert(String),
    Delete(Motion<'a>),
    Undo,
    Redo,
    DeleteSelected,
    Move(Motion<'a>),
    /// Expand or change the selection
    Selection(Motion<'a>),
    /// Create a new cursor at the location the motion targets
    NewCaret(Motion<'a>),
}

/// A motion, either character-wise or defined by some higher-level semantic target.
/// Conceptually similar to motions in vim (`w`, `t$`)
#[allow(unused)]
#[derive(Debug, Copy, Clone)]
pub(crate) enum Motion<'a> {
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
    FindNext(&'a hotsauce::Regex),
    FindPrev(&'a hotsauce::Regex),
}
