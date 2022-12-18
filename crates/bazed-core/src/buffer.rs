use std::collections::HashMap;

use nonempty::{nonempty, NonEmpty};
use tap::Pipe;
use xi_rope::{engine::Engine, tree::NodeInfo, DeltaBuilder, Rope, RopeDelta, Transformer};

use crate::region::{Region, RegionId};

/// Stores all the active regions in a buffer.
///
/// Terminology:
/// - *Cursor* refers to any region with length 0
/// - *Caret* refers to cursors that represent concrete, user-controlled cursors.
///   (i.e.: The places where text gets inserted)
struct BufferRegions {
    regions: HashMap<RegionId, Region>,
    /// All the active carets. There will always be at least one.
    /// The first element may be considered the "primary" caret,
    /// being the caret that will remain when exiting any sort of multi-caret mode.
    ///
    /// All possible mutating interactions with [BufferRegions] must guarantee
    /// that all ids stored here continue to actually map to a region.
    carets: NonEmpty<RegionId>,
}

impl Default for BufferRegions {
    fn default() -> Self {
        let primary_caret = Region::cursor(0);
        let primary_caret_id = RegionId::gen();
        let regions = maplit::hashmap! { primary_caret_id => primary_caret };
        let carets = nonempty![primary_caret_id];
        Self { regions, carets }
    }
}

impl BufferRegions {
    fn apply_transformer<N: NodeInfo>(&mut self, trans: &mut Transformer<N>) {
        for region in self.regions.values_mut() {
            region.apply_transformer(trans, false);
        }
    }

    fn apply_delta(&mut self, delta: &RopeDelta) {
        let mut transformer = xi_rope::Transformer::new(delta);
        self.apply_transformer(&mut transformer);
    }

    fn carets(&self) -> NonEmpty<Region> {
        self.carets
            .iter()
            .map(|x| self.regions.get(x).expect("caret not found in regions"))
            .cloned()
            .collect::<Vec<_>>()
            .pipe(NonEmpty::from_vec)
            .unwrap()
    }
}

pub struct Buffer {
    text: Rope,
    engine: Engine,
    undo_group_id: usize,
    regions: BufferRegions,
}

impl Buffer {
    pub fn open_ephemeral() -> Self {
        let rope = Rope::from(String::new());
        Self {
            undo_group_id: 1,
            engine: Engine::new(rope.clone()),
            text: rope,
            regions: BufferRegions::default(),
        }
    }

    pub fn content_to_string(&self) -> String {
        self.engine.get_head().to_string()
    }

    #[tracing::instrument(skip(self))]
    pub fn commit_delta(&mut self, delta: RopeDelta) -> Rope {
        let head_rev = self.engine.get_head_rev_id();
        let undo_group = self.calculate_undo_group();
        //self.last_edit_type = self.this_edit_type;

        self.regions.apply_delta(&delta);
        self.engine.edit_rev(1, undo_group, head_rev.token(), delta);

        self.text = self.engine.get_head().clone();
        self.text.clone()
    }

    fn calculate_undo_group(&mut self) -> usize {
        // TODO Currently this just creates a new undo group every time.
        // in the future, we should possibly create undo groups based
        // on edit types that belong together (i.e. insert, delete, etc).
        // this would mean that consecutive edits of the same kind,
        // will get merged into the same undo group.
        self.undo_group_id += 1;
        self.undo_group_id
    }

    pub fn insert_at_primary(&mut self, chars: &str) {
        self.do_insert(self.regions.carets(), chars)
    }

    pub fn do_insert(&mut self, regions: impl IntoIterator<Item = Region>, chars: &str) {
        // This is also where xi handles surrounding stuff in parens when something is selected.
        // i.e. when the text "foo" is in the selection, and the chars are "(",
        // then this would turn the text into "(foo)"
        // We don't yet handle this at all, and I'm not sure if we want to.

        let mut builder = DeltaBuilder::new(self.text.len());
        let text: Rope = chars.into();
        for region in regions {
            builder.replace(region, text.clone());
        }
        let delta = builder.build();
        self.commit_delta(delta);
    }
}
