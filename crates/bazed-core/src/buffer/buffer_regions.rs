use std::collections::HashMap;

use nonempty::NonEmpty;
use tap::Pipe;
use xi_rope::{tree::NodeInfo, RopeDelta, Transformer};

use crate::region::{Region, RegionId};

/// Stores all the active regions in a buffer.
///
/// Terminology:
/// - *Region* refers to any region in the buffer
/// - *Cursor* refers to any region of length 0
/// - *Selection* refers to regions that represent concrete, user-controlled selections
/// - *Caret* refers to regions that represent concrete, user-controlled carets.
///   (i.e.: The places where text gets inserted)
///   Currently this also includes selections.
#[derive(Debug)]
pub(super) struct BufferRegions {
    regions: HashMap<RegionId, Region>,
    /// All the active carets, including the primary caret.
    /// This list needs to be kept ordered and non-overlapping.
    ///
    /// All possible mutating interactions with [BufferRegions] must guarantee
    /// that all ids stored here continue to actually map to a region.
    carets: Vec<RegionId>,
    /// The primary caret is the caret that will remain when exiting any sort of multi-caret mode.
    primary_caret_id: RegionId,
}

impl Default for BufferRegions {
    fn default() -> Self {
        let primary_caret = Region::sticky_cursor(0);
        let primary_caret_id = RegionId::gen();
        let regions = maplit::hashmap! { primary_caret_id => primary_caret };
        Self {
            regions,
            carets: vec![primary_caret_id],
            primary_caret_id,
        }
    }
}

impl BufferRegions {
    pub(super) fn apply_transformer<N: NodeInfo>(&mut self, trans: &mut Transformer<N>) {
        for region in self.regions.values_mut() {
            region.apply_transformer(trans);
        }
        self.make_carets_consistent();
    }

    pub(super) fn apply_delta(&mut self, delta: &RopeDelta) {
        let mut transformer = xi_rope::Transformer::new(delta);
        self.apply_transformer(&mut transformer);
    }

    /// Return all carets in this buffer. Guaranteed to be ordered and non-overlapping
    pub(super) fn carets(&self) -> NonEmpty<Region> {
        let carets = self
            .carets
            .iter()
            .map(|x| *self.regions.get(x).expect("caret not found in region"))
            .collect::<Vec<_>>()
            .pipe(NonEmpty::from_vec)
            .unwrap();
        debug_assert!(
            carets
                .iter()
                .zip(carets.iter().skip(1))
                .all(|(a, b)| a.is_strictly_before(*b)),
            "Caret list was not strictly ordered"
        );
        carets
    }

    pub(super) fn update_regions<F>(&mut self, mut f: F)
    where
        F: FnMut(&RegionId, &mut Region),
    {
        for (id, region) in self.regions.iter_mut() {
            f(id, region);
        }
        self.make_carets_consistent()
    }

    pub(super) fn update_carets<F>(&mut self, mut f: F)
    where
        F: FnMut(&RegionId, &mut Region),
    {
        for id in &self.carets {
            f(id, self.regions.get_mut(id).unwrap());
        }
        self.make_carets_consistent()
    }

    /// Add a new caret and return the generated id.
    ///
    /// Note that the caret may imediately get merged into another region.
    pub(super) fn add_caret(&mut self, make_primary: bool, region: Region) {
        let id = RegionId::gen();
        self.carets.push(id);
        self.regions.insert(id, region);
        if make_primary {
            self.primary_caret_id = id;
        }
        self.make_carets_consistent();
    }

    /// Directly overwrite the primary caret / selection.
    /// **Note** that you should ensure you're always setting a sticky region here.
    pub(super) fn set_primary_caret(&mut self, region: Region) {
        self.regions.insert(self.primary_caret_id, region);
        self.make_carets_consistent();
    }

    /// Ensure that the list of carets is ordered and carets are not overlapping.
    ///
    /// TODO For now, we just run this after any change to the regions, which is obviously suboptimal
    /// but should be good enough for now.
    /// In the long run, we should probably just reposition the elements that have changed, rather
    /// than sorting everything all the time.
    fn make_carets_consistent(&mut self) {
        // Sort the regions
        self.carets
            .sort_unstable_by_key(|id| self.regions.get(id).unwrap().head);
        // Then merge overlapping regions
        let mut i = 0;
        while i < (self.carets.len() - 1) {
            let caret_b = self.regions.get(&self.carets[i + 1]).unwrap().clone();
            let caret_a = self.regions.get_mut(&self.carets[i]).unwrap();
            if caret_a.range().end >= caret_b.range().start {
                // TODO I'm not sure if this is the behavior we want,
                // as this moves the head end of the region when merging
                // with a region on its right.. but also, this should never really happen.
                *caret_a = caret_a.with_end_at(caret_b.range().end);
                let deleted_caret = self.carets.remove(i + 1);
                if self.primary_caret_id == deleted_caret {
                    // If we just removed the primary caret, make caret that resulted from the merge
                    // the primary one
                    self.primary_caret_id = self.carets[i];
                }
            } else {
                i += 1;
            }
        }
    }

    pub(super) fn collapse_selections(&mut self) {
        self.update_carets(|_, c| {
            c.tail = c.head;
        });
    }
    pub(super) fn collapse_carets_into_primary(&mut self) {
        for id in self.carets.drain(..) {
            if id != self.primary_caret_id {
                self.regions.remove(&id);
            }
        }
        self.carets.push(self.primary_caret_id);
    }
}
