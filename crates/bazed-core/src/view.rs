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

/// A view represents a part of a [crate::buffer::Buffer] that is shown by a client.
pub struct View {
    /// Id of the [crate::document::Document] this view looks into
    pub document_id: DocumentId,
    /// Viewport of this view
    pub vp: Viewport,
}

impl View {
    pub fn new(document_id: DocumentId, viewport: Viewport) -> Self {
        Self {
            document_id,
            vp: viewport,
        }
    }
}

/// Information about which part of a [crate::buffer::Buffer] is visible to the client.
/// Currently only vertical position and height is considered.
pub struct Viewport {
    /// Index of the first line shown in the viewport
    pub first_line: usize,
    /// Number of lines shown in the viewport
    pub height: usize,
}

impl Viewport {
    pub fn new(first_line: usize, height: usize) -> Self {
        Self { first_line, height }
    }

    /// Create a viewport starting at line 0 and going down 100 million lines.
    #[cfg(test)]
    pub fn new_ginormeous() -> Self {
        Self {
            first_line: 0,
            height: 100_000_000,
        }
    }
    /// first and last line shown in the viewport
    pub fn last_line(&self) -> usize {
        self.first_line + self.height
    }
}
