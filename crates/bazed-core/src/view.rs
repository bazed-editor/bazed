use uuid::Uuid;

use crate::document::DocumentId;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, derive_more::Display, derive_more::Into)]
pub struct ViewId(Uuid);

impl ViewId {
    pub(crate) fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    pub(crate) fn gen() -> Self {
        Self(Uuid::new_v4())
    }
}

// TODO this will need to also account for variable-width fonts, ligatures as well as tab characters in the future.

/// A view represents a part of a [Buffer] that is shown by a client.
/// It stores information about the viewport in terms of lines and columns,
/// and thus currently assumes a monospaced font.
pub struct View {
    /// Index of the first line shown in the viewport
    pub first_line: usize,
    /// Index of the first column shown in the viewport
    pub first_col: usize,
    /// Number of lines shown in the viewport
    pub height: usize,
    /// Number of columns shown in the viewport
    pub width: usize,
    /// Id of the [Document] this view looks into
    pub document_id: DocumentId,
}

impl View {
    pub fn new(document_id: DocumentId, height: usize, width: usize) -> Self {
        Self {
            first_line: 0,
            first_col: 0,
            height,
            width,
            document_id,
        }
    }

    /// first and last line shown in the viewport
    pub fn last_line(&self) -> usize {
        self.first_line + self.height
    }

    /// first and last column shown in the viewport
    pub fn last_col(&self) -> usize {
        self.first_col + self.width
    }
}
