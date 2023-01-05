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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    /// last line shown in the viewport
    pub fn last_line(&self) -> usize {
        (self.first_line + self.height).saturating_sub(1)
    }

    /// Move the viewport such that the given line_nr is in view, and
    /// attempt to keep it a minimum of `scroll_off` lines from the viewport edges
    /// as long as we're not at the start of the file
    pub fn with_line_in_view(&self, line_nr: usize, scroll_off: usize) -> Self {
        let mut vp = self.clone();
        vp.first_line = if line_nr < vp.first_line + scroll_off {
            line_nr.saturating_sub(scroll_off)
        } else if line_nr > vp.last_line().saturating_sub(scroll_off) {
            let new_last_line = line_nr + scroll_off;
            new_last_line.saturating_sub(vp.height - 1)
        } else {
            vp.first_line
        };
        vp
    }
}

#[cfg(test)]
mod test {
    use super::Viewport;
    use crate::test_util;

    #[test]
    fn test_scroll_line_into_view() {
        test_util::setup_test();
        assert_eq!(
            Viewport::new(0, 10),
            Viewport::new(0, 10).with_line_in_view(0, 0)
        );
        assert_eq!(
            Viewport::new(0, 10),
            Viewport::new(2, 10).with_line_in_view(2, 2)
        );
        assert_eq!(
            Viewport::new(5, 10),
            Viewport::new(2, 10).with_line_in_view(12, 2)
        );
    }
}
