use std::ops::Range;

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
    /// The column this region "wants" to be at.
    /// Used to remember horizontal position when moving across shorter lines
    pub preferred_column: Option<usize>,
}

impl Region {
    pub fn sticky_cursor(offset: usize) -> Self {
        Self::sticky(offset, offset)
    }

    pub fn sticky(head: usize, tail: usize) -> Self {
        Self {
            head,
            tail,
            stickyness: Stickyness::Sticky,
            preferred_column: None,
        }
    }

    /// Set the end (the head or the tail, depending on what offset is higher) to the given offset.
    pub fn with_end_at(mut self, offset: usize) -> Self {
        if self.head <= self.tail {
            self.tail = offset;
        } else {
            self.head = offset;
        }
        self
    }

    /// Check if  this Region represents a cursor, meaning that it has length 0
    pub fn is_cursor(&self) -> bool {
        self.head == self.tail
    }

    pub fn apply_transformer<N: xi_rope::tree::NodeInfo>(
        &mut self,
        transformer: &mut xi_rope::Transformer<N>,
    ) {
        self.head = transformer.transform(self.head, self.stickyness == Stickyness::Sticky);
        self.tail = transformer.transform(self.tail, self.stickyness == Stickyness::Sticky);
    }

    /// Return the head and tail in order of offset
    pub fn range(&self) -> Range<usize> {
        if self.head <= self.tail {
            self.head..self.tail
        } else {
            self.tail..self.head
        }
    }

    /// Check if this range overlaps with another range.
    pub fn overlaps(&self, other: Self) -> bool {
        let a_range = self.range();
        let b_range = other.range();
        a_range.start <= b_range.end && b_range.start <= a_range.end
    }

    /// Check if this region is before another one without any overlap.
    pub fn is_strictly_before(&self, other: Self) -> bool {
        self.head < other.head && !self.overlaps(other)
    }

    /// Merge this region into another region. Keeps all non-positional attributes of `self`.
    /// Returns none when the regions did not overlap
    pub fn merge(&self, other: Self) -> Option<Self> {
        if !self.overlaps(other) {
            return None;
        }
        let own_range = self.range();
        let other_range = other.range();
        Some(Self {
            head: usize::min(own_range.start, other_range.start),
            tail: usize::max(own_range.end, other_range.end),
            ..*self
        })
    }
}

impl IntervalBounds for Region {
    fn into_interval(self, _upper_bound: usize) -> xi_rope::Interval {
        self.range().into()
    }
}

#[cfg(test)]
mod test {
    use super::Region;

    #[test]
    fn test_overlaps() {
        let check_works_both_ways = |a: Region, b: Region| {
            assert!(a.overlaps(b), "A overlaps B");
            assert!(b.overlaps(a), "B overlaps A");
        };
        check_works_both_ways(Region::sticky(10, 20), Region::sticky(15, 16));
        check_works_both_ways(Region::sticky(10, 20), Region::sticky(5, 15));
        check_works_both_ways(Region::sticky(10, 20), Region::sticky(15, 20));
        check_works_both_ways(Region::sticky(10, 20), Region::sticky(15, 15));

        assert!(Region::sticky(10, 20).overlaps(Region::sticky(20, 25)));
        assert!(Region::sticky(20, 25).overlaps(Region::sticky(10, 20)));
    }

    #[test]
    fn test_merge() {
        assert_eq!(
            Some(Region::sticky(10, 20)),
            Region::sticky(10, 15).merge(Region::sticky(12, 20))
        );
        assert_eq!(None, Region::sticky(10, 15).merge(Region::sticky(18, 20)));
    }
}
