use std::collections::HashMap;

use xi_rope::{engine::Engine, DeltaBuilder, Rope, RopeDelta};

use crate::region::{Region, RegionId};

pub struct Buffer {
    text: Rope,
    engine: Engine,
    dirty: bool,
    undo_group_id: usize,
    regions: HashMap<RegionId, Region>,
    primary_cursor: RegionId,
}

impl Buffer {
    pub fn open_ephemeral() -> Self {
        let rope = Rope::from(String::new());
        let primary_cursor = RegionId::gen();
        let mut regions = HashMap::new();
        regions.insert(primary_cursor, Region::cursor(0));
        Self {
            dirty: false,
            undo_group_id: 1,
            engine: Engine::new(rope.clone()),
            text: rope,
            regions,
            primary_cursor,
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    #[tracing::instrument(skip(self))]
    pub fn commit_delta(&mut self, delta: RopeDelta) -> Rope {
        let head_rev_id = self.engine.get_head_rev_id();
        let undo_group = self.calculate_undo_group();
        //self.last_edit_type = self.this_edit_type;

        let mut transformer = xi_rope::Transformer::new(&delta);
        for region in self.regions.values_mut() {
            region.apply_transformer(&mut transformer, false);
        }
        self.engine
            .edit_rev(1, undo_group, head_rev_id.token(), delta);

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
        let region = self
            .regions
            .get(&self.primary_cursor)
            .expect("Primary cursor not found in regions");
        self.do_insert(&[*region], chars)
    }

    pub fn do_insert(&mut self, regions: &[Region], chars: &str) {
        // This is also where xi handles surrounding stuff in parens when something is selected.
        // i.e. when the text "foo" is in the selection, and the chars are "(",
        // then this would turn the text into "(foo)"
        // We don't yet handle this at all, and I'm not sure if we want to.

        let mut builder = DeltaBuilder::new(self.text.len());
        let text: Rope = chars.into();
        for region in regions {
            builder.replace(*region, text.clone());
        }
        let delta = builder.build();
        self.commit_delta(delta);
    }

    pub fn content_to_string(&self) -> String {
        self.engine.get_head().to_string()
    }
}
