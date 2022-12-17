use uuid::Uuid;
use xi_rope::interval::IntervalBounds;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, derive_more::Display, derive_more::Into)]
pub struct RegionId(Uuid);

impl RegionId {
    pub(crate) fn gen() -> Self {
        RegionId(Uuid::new_v4())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord, derive_more::Display)]
#[display(fmt = "{start}..{end}")]
pub struct Region {
    start: usize,
    end: usize,
}

impl Region {
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end);
        Self { start, end }
    }

    pub fn caret(at: usize) -> Self {
        Self::new(at, at)
    }

    pub fn apply_transformer<N: xi_rope::tree::NodeInfo>(
        &mut self,
        transformer: &mut xi_rope::Transformer<N>,
        after: bool,
    ) {
        self.start = transformer.transform(self.start, after);
        self.end = transformer.transform(self.end, after);
    }

    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }

    /// A region is considered a caret if its length is 0
    pub fn is_caret(&self) -> bool {
        self.start == self.end
    }
}

impl IntervalBounds for Region {
    fn into_interval(self, _upper_bound: usize) -> xi_rope::Interval {
        xi_rope::Interval {
            start: self.start,
            end: self.end,
        }
    }
}
