use std::collections::{BTreeSet, HashMap};

use nonempty::{nonempty, NonEmpty};
use tap::Pipe;
use xi_rope::{engine::Engine, tree::NodeInfo, DeltaBuilder, Rope, RopeDelta, Transformer};

use crate::{
    region::{Region, RegionId},
    user_buffer_op::{BufferOp, EditOp, MovementOp},
};

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
            region.apply_transformer(trans, true);
        }
    }

    fn apply_delta(&mut self, delta: &RopeDelta) {
        let mut transformer = xi_rope::Transformer::new(delta);
        self.apply_transformer(&mut transformer);
    }

    fn carets(&self) -> NonEmpty<Region> {
        self.carets
            .iter()
            .map(|x| *self.regions.get(x).expect("caret not found in regions"))
            .collect::<Vec<_>>()
            .pipe(NonEmpty::from_vec)
            .unwrap()
    }

    fn carets_mut(&mut self) -> impl Iterator<Item = &mut Region> {
        // TODO This is stupid, but iterating over self.carets instead and getting the refs
        // through get_mut doesn't work trivially, as rust can't verify that we won't get multiple
        // mut refs to the same entry as a result of overlapping keys...
        self.regions
            .iter_mut()
            .filter(|(k, _)| self.carets.contains(k))
            .map(|(_, v)| v)
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
    fn commit_delta(&mut self, delta: RopeDelta) -> Rope {
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

    fn insert_at_carets(&mut self, chars: &str) {
        // This is also where xi handles surrounding stuff in parens when something is selected.
        // i.e. when the text "foo" is in the selection, and the chars are "(",
        // then this would turn the text into "(foo)"
        // We don't yet handle this at all, and I'm not sure if we want to.

        let mut builder = DeltaBuilder::new(self.text.len());
        let text: Rope = chars.into();
        for region in self.regions.carets() {
            builder.replace(region, text.clone());
        }
        let delta = builder.build();
        self.commit_delta(delta);
    }

    fn delete_backward_at_carets(&mut self) {
        let mut builder = DeltaBuilder::new(self.text.len());
        for region in self.regions.carets() {
            // See xi-editors `offset_for_delete_backwards` function in backward.rs...
            // all I'll say is `#[allow(clippy::cognitive_complexity)]`.
            let delete_start = 1.max(region.start) - 1;
            builder.delete(Region::new(delete_start, region.end));
        }
        let delta = builder.build();
        self.commit_delta(delta);
    }

    fn undo(&mut self) {
        if self.undo_group_id > 1 {
            let mut undos = BTreeSet::new();
            undos.insert(self.undo_group_id);
            self.undo_group_id -= 1;
            self.engine.undo(undos);
            self.text = self.engine.get_head().clone();
        }
    }

    pub(crate) fn apply_buffer_op(&mut self, op: BufferOp) {
        match op {
            BufferOp::Edit(x) => self.apply_edit_op(x),
            BufferOp::Movement(x) => self.apply_movement_op(x),
        }
    }

    pub(crate) fn apply_edit_op(&mut self, op: EditOp) {
        match op {
            EditOp::Insert(text) => self.insert_at_carets(&text),
            EditOp::Backspace => self.delete_backward_at_carets(),
            EditOp::Undo => self.undo(),
        }
    }

    pub(crate) fn apply_movement_op(&mut self, op: MovementOp) {
        for caret in self.regions.carets_mut() {
            *caret = apply_movement_to_cursor(*caret, op, &self.text);
        }
    }
}

fn apply_movement_to_cursor(region: Region, op: MovementOp, text: &Rope) -> Region {
    assert!(
        region.is_cursor(),
        "Movement for non-cursor regions is not implemented yet, and I'm not sure how to best approach this"
    );
    let offset = match op {
        MovementOp::Left => text
            .prev_grapheme_offset(region.start)
            .unwrap_or(region.start),
        MovementOp::Right => text
            .next_grapheme_offset(region.start)
            .unwrap_or(region.start),
        MovementOp::Up => unimplemented!("Vertical movement"),
        MovementOp::Down => unimplemented!("Vertical movement"),
    };
    Region::cursor(offset)
}
