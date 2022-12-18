use uuid::Uuid;
use xi_rope::interval::IntervalBounds;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, derive_more::Display, derive_more::Into)]
pub struct MarkId(Uuid);

impl MarkId {
    pub(crate) fn gen() -> Self {
        MarkId(Uuid::new_v4())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, derive_more::Display, Default)]
pub enum MarkKind {
    #[default]
    Sticky,
    NonSticky,
}

/// A mark represents a known position in a buffer, that will move when the text around it moves.
#[derive(Debug, Eq, PartialEq, Clone, Copy, derive_more::Display, Default)]
#[display(fmt = "{offset}")]
pub struct Mark {
    pub offset: usize,
    pub kind: MarkKind,
}

impl Mark {
    pub fn sticky(offset: usize) -> Self {
        Self {
            offset,
            kind: MarkKind::Sticky,
        }
    }

    pub fn non_sticky(offset: usize) -> Self {
        Self {
            offset,
            kind: MarkKind::NonSticky,
        }
    }

    pub fn apply_transformer<N: xi_rope::tree::NodeInfo>(
        &mut self,
        transformer: &mut xi_rope::Transformer<N>,
    ) {
        self.offset = transformer.transform(self.offset, self.kind == MarkKind::Sticky);
    }
}

impl IntervalBounds for Mark {
    fn into_interval(self, _upper_bound: usize) -> xi_rope::Interval {
        xi_rope::Interval {
            start: self.offset,
            end: self.offset,
        }
    }
}
