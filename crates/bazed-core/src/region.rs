use uuid::Uuid;
use xi_rope::interval::IntervalBounds;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, derive_more::Display, derive_more::Into)]
pub struct RegionId(Uuid);

impl RegionId {
    pub(crate) fn gen() -> Self {
        RegionId(Uuid::new_v4())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default, derive_more::Display)]
pub enum Stickyness {
    #[default]
    Sticky,
    NonSticky,
}

/// A region represents a range in a buffer that will move when the text around it moves.
/// This is useful for representing cursors, selections or any other positions within a piece of text.
///
/// head and `tail` represent absolute positions in the text, and are not necessarily ordered.
/// For cursors and selections, the `head` should be considered the part that
/// is moved by the user when extending selections.
///
/// Regions are defined by offsets in between characters,
/// which means that a region `<2..4>` includes the characters at index 2 and 3.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Default, derive_more::Display)]
#[display(fmt = "<{head}..{tail}>")]
pub struct Region {
    pub head: usize,
    pub tail: usize,
    pub stickyness: Stickyness,
}

impl Region {
    pub fn sticky_cursor(offset: usize) -> Self {
        Self {
            head: offset,
            tail: offset,
            stickyness: Stickyness::Sticky,
        }
    }

    pub fn apply_transformer<N: xi_rope::tree::NodeInfo>(
        &mut self,
        transformer: &mut xi_rope::Transformer<N>,
    ) {
        self.head = transformer.transform(self.head, self.stickyness == Stickyness::Sticky);
        self.tail = transformer.transform(self.tail, self.stickyness == Stickyness::Sticky);
    }

    /// Return the head and tail in order of offset
    pub fn range(&self) -> (usize, usize) {
        if self.head <= self.tail {
            (self.head, self.tail)
        } else {
            (self.tail, self.head)
        }
    }
}

impl IntervalBounds for Region {
    fn into_interval(self, _upper_bound: usize) -> xi_rope::Interval {
        let (start, end) = self.range();
        xi_rope::Interval { start, end }
    }
}
