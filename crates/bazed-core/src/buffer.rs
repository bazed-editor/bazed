//! Buffer manages all editing operations on the text structure,
//! including adjusting carets and selections when edits happen and keeping an undo-history.
//!
//! # Some terminology
//! - **offset**: a point in between two code points in the text,
//!   or the points directly before the first and directly behind the last one.
//!   Note that this implies that `text.len()` is a valid offset in `text`.
//!
//! Terminology of `Region`s and `Carets` etc. is specified in [BufferRegions].

use nonempty::NonEmpty;
use xi_rope::{engine::Engine, DeltaBuilder, Rope, RopeDelta};

use self::{buffer_regions::BufferRegions, position::Position, undo_history::UndoHistory};
use crate::{
    region::Region,
    user_buffer_op::{BufferOp, EditType, Motion, Trajectory},
    view::Viewport,
    word_boundary,
};

mod buffer_regions;
pub mod position;
mod undo_history;

#[derive(Debug)]
pub struct Buffer {
    text: Rope,
    engine: Engine,
    regions: BufferRegions,
    undo_history: UndoHistory,
    /// edit type of the most recently performed action, kept for grouping edits into undo-groups
    last_edit_type: EditType,
}

impl Buffer {
    pub fn new_from_string(s: String) -> Self {
        let rope = Rope::from(s);
        Self {
            engine: Engine::new(rope.clone()),
            text: rope,
            regions: BufferRegions::default(),
            undo_history: UndoHistory::default(),
            last_edit_type: EditType::Other,
        }
    }

    pub fn new_empty() -> Self {
        Self::new_from_string(String::new())
    }

    pub fn content_to_string(&self) -> String {
        self.engine.get_head().to_string()
    }

    /// Return a snapshot of the latest commited state of the text
    pub fn head_rope(&self) -> &Rope {
        self.engine.get_head()
    }

    pub fn all_caret_positions(&self) -> NonEmpty<Position> {
        self.regions.carets().map(|x| {
            Position::from_offset(&self.text, x.head)
                .expect("Caret stored in BufferRegions was not a valid offset into the buffer")
        })
    }

    /// get the lines in the given inclusive range
    pub fn lines_between(
        &self,
        low: usize,
        high: usize,
    ) -> impl Iterator<Item = std::borrow::Cow<str>> {
        // TODO lines takes a range, so this is probably a very bad way of doing this...
        // let's look into optimizing this.
        self.text.lines(..).skip(low).take(high - low)
    }

    /// Snap all regions to the closest valid points in the buffer.
    ///
    /// This may be required if an action (such as undo, currently) changes the buffer
    /// without moving the regions accordingly. In the future, this should not be required
    /// as all actions _should_ move all regions properly, either through a coordinate transform
    /// with [xi_rope::Transformer], or, in the case of undo, by remembering where the carets where before.
    ///
    /// **WARNING:** This is very much a temporary solution, as it _will_ cause inconsistent state as soon as we use
    /// regions for more than just caret position. (see https://github.com/bazed-editor/bazed/issues/47)
    fn snap_regions_to_valid_position(&mut self) {
        self.regions.update_regions(|_, region| {
            region.head = region.head.min(self.text.len());
            region.tail = region.tail.min(self.text.len());
        });
    }

    #[tracing::instrument(skip(self), fields(head_rev_id = ?self.engine.get_head_rev_id()))]
    fn commit_delta(&mut self, delta: RopeDelta, edit_type: EditType) -> Rope {
        tracing::debug!("Committing delta");
        self.regions.apply_delta(&delta);

        if self.last_edit_type != edit_type {
            self.undo_history.start_new_undo_group();
        }
        let undo_group = self.undo_history.calculate_undo_id();
        tracing::trace!(undo_group, "determined undo group id");
        self.last_edit_type = edit_type;

        let head_rev = self.engine.get_head_rev_id();
        self.engine.edit_rev(1, undo_group, head_rev.token(), delta);

        self.text = self.engine.get_head().clone();
        self.text.clone()
    }

    fn insert_at_carets(&mut self, chars: &str) {
        let mut builder = DeltaBuilder::new(self.text.len());
        let text: Rope = chars.into();
        tracing::debug!(
            "Inserting, caret regions are: {:?}",
            self.regions.carets().iter().collect::<Vec<_>>()
        );
        for region in self.regions.carets() {
            builder.replace(region, text.clone());
        }
        let delta = builder.build();
        self.commit_delta(delta, EditType::Insert);
    }

    fn delete_at_carets(&mut self, traj: Trajectory) {
        let mut builder = DeltaBuilder::new(self.text.len());
        for region in self.regions.carets() {
            // See xi-editors `offset_for_delete_backwards` function in backward.rs...
            // all I'll say is `#[allow(clippy::cognitive_complexity)]`.
            let range = match traj {
                Trajectory::Forwards => region.head..self.text.len().min(region.head + 1),
                Trajectory::Backwards => (1.max(region.head) - 1)..region.head,
            };
            builder.delete(range);
        }
        let delta = builder.build();
        self.commit_delta(delta, EditType::Delete);
    }

    fn undo(&mut self) {
        tracing::trace!(
            history = ?self.undo_history,
            head_rev_id = ?self.engine.get_head_rev_id(),
            "before undo",
        );
        if self.undo_history.undo() {
            self.last_edit_type = EditType::Other;

            let old_head_rev = self.engine.get_head_rev_id();

            self.engine
                .undo(self.undo_history.currently_undone().clone());
            self.text = self.engine.get_head().clone();

            match self.engine.try_delta_rev_head(old_head_rev.token()) {
                Ok(delta) => self.regions.apply_delta(&delta),
                Err(err) => {
                    tracing::error!("Error generating delta after undo: {err}");
                    self.snap_regions_to_valid_position();
                },
            }
        }
        tracing::trace!(
            history = ?self.undo_history,
            head_rev_id = ?self.engine.get_head_rev_id(),
            "after undo",
        );
    }

    fn redo(&mut self) {
        tracing::trace!(history = ?self.undo_history, "before redo");
        if self.undo_history.redo() {
            self.last_edit_type = EditType::Other;
            let old_head_rev = self.engine.get_head_rev_id();

            self.engine
                .undo(self.undo_history.currently_undone().clone());
            self.text = self.engine.get_head().clone();

            match self.engine.try_delta_rev_head(old_head_rev.token()) {
                Ok(delta) => self.regions.apply_delta(&delta),
                Err(err) => {
                    tracing::error!("Error generating delta after redo: {err}");
                    self.snap_regions_to_valid_position();
                },
            }
        }
        tracing::trace!(history = ?self.undo_history, "after redo");
    }

    /// Jump the user caret to a given position.
    ///
    /// If `snap` is true,
    /// we'll snap the cursor to the first valid offset in the given line and to the closest valid line.
    /// If `snap` is false andthere is no text at the given position, we'll do nothing.
    ///
    /// If there is more than one caret, collapses all carets down into the main one.
    ///
    /// Returns true if the caret has changed, false otherwise
    pub fn jump_caret_to_position(&mut self, coords: Position, snap: bool) -> bool {
        let offset = if snap {
            coords.to_offset(&self.text)
        } else {
            Some(coords.to_offset_snapping(&self.text))
        };
        if let Some(offset) = offset {
            self.regions.collapse_carets_into_primary();
            self.regions
                .set_primary_caret(Region::sticky_cursor(offset));
            true
        } else {
            false
        }
    }

    pub(crate) fn apply_buffer_op(&mut self, vp: &Viewport, op: BufferOp) {
        // TODO How should _any_ of these behave when there is a selection?
        // Insertion should replace, backspace should delete, etc. How do we implement that cleanly?
        match op {
            BufferOp::Insert(text) => self.insert_at_carets(&text),
            BufferOp::Delete(traj) => self.delete_at_carets(traj),
            BufferOp::Undo => self.undo(),
            BufferOp::Redo => self.redo(),
            BufferOp::Move(motion) => {
                // TODO is this the strat?
                // Do we just discard selections when moving without BufferOp::Selection?
                self.move_carets(vp, motion);
            },
            BufferOp::Selection(motion) => self.regions.update_carets(|_, region| {
                *region = apply_motion_to_region(&self.text, vp, *region, true, motion);
            }),
            BufferOp::NewCaret(motion) => {
                let carets = self.regions.carets();
                let primary_caret = carets.first();
                let new_caret =
                    apply_motion_to_region(&self.text, vp, *primary_caret, false, motion);
                if &new_caret != primary_caret {
                    self.regions.add_caret(true, new_caret);
                }
            },
        }
    }

    /// Move carets by a given motion, collapsing any selections down into carets.
    pub(crate) fn move_carets(&mut self, viewport: &Viewport, motion: Motion) {
        self.regions.update_carets(|_, region| {
            *region = apply_motion_to_region(&self.text, viewport, *region, false, motion);
        })
    }
}

/// Apply a given motion to a region.
/// if `only_move_head` is false, the tail of the region gets set to the new head,
/// collapsing it into a cursor.
///
/// May result in a region at offset `text.len()`, meaning that it is outside the bounds of the text.
fn apply_motion_to_region(
    text: &Rope,
    vp: &Viewport,
    region: Region,
    only_move_head: bool,
    motion: Motion,
) -> Region {
    // The column the new region wants to be in
    // set when moving vertically, for use when coming out of a shorter line.
    let mut preferred_column = None;
    let offset = match motion {
        Motion::Left => text
            .prev_grapheme_offset(region.head)
            .unwrap_or(region.head),
        Motion::Right => text
            .next_grapheme_offset(region.head)
            .unwrap_or(region.head),
        Motion::StartOfLine => {
            let line = text.line_of_offset(region.head);
            text.offset_of_line(line)
        },
        Motion::EndOfLine => {
            let line = text.line_of_offset(region.head);
            let last_line = text.line_of_offset(text.len());
            if line < last_line {
                text.offset_of_line(line + 1)
            } else {
                text.len()
            }
        },
        Motion::NextWordBoundary(boundary_type) => {
            word_boundary::find_word_boundaries(text, region.head)
                .filter(|(_, t)| t.matches(&boundary_type))
                .next()
                .map_or(text.len(), |(offset, _)| offset)
        },
        Motion::PrevWordBoundary(boundary_type) => {
            word_boundary::find_word_boundaries_backwards(text, region.head)
                .filter(|(_, t)| t.matches(&boundary_type))
                .next()
                .map_or(0, |(offset, _)| offset)
        },

        Motion::Up => {
            let pos = Position::from_offset(text, region.head).unwrap();
            let pos = match region.preferred_column {
                Some(cur_preferred_column) => pos.with_col(cur_preferred_column),
                None => pos,
            };
            preferred_column = Some(pos.col);
            if pos.line > 0 {
                pos.with_line(pos.line - 1).to_offset_snapping(text)
            } else {
                region.head
            }
        },
        Motion::Down => {
            let pos = Position::from_offset(text, region.head).unwrap();
            let pos = match region.preferred_column {
                Some(cur_preferred_column) => pos.with_col(cur_preferred_column),
                None => pos,
            };
            preferred_column = Some(pos.col);
            let last_line = text.line_of_offset(text.len());
            if pos.line < last_line {
                pos.with_line(pos.line + 1).to_offset_snapping(text)
            } else {
                region.head
            }
        },
        Motion::TopOfViewport => {
            let pos = Position::from_offset(text, region.head).unwrap();
            let pos = match region.preferred_column {
                Some(cur_preferred_column) => pos.with_col(cur_preferred_column),
                None => pos,
            };
            preferred_column = Some(pos.col);
            pos.with_line(vp.first_line).to_offset_snapping(text)
        },
        Motion::BottomOfViewport => {
            let pos = Position::from_offset(text, region.head).unwrap();
            let pos = match region.preferred_column {
                Some(cur_preferred_column) => pos.with_col(cur_preferred_column),
                None => pos,
            };
            preferred_column = Some(pos.col);
            let last_line = text.line_of_offset(text.len());
            let target_line = vp.last_line().min(last_line);
            pos.with_line(target_line).to_offset_snapping(text)
        },
    };

    Region {
        head: offset,
        tail: if only_move_head { region.tail } else { offset },
        stickyness: region.stickyness,
        preferred_column,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util;
    use crate::view::Viewport;
    use crate::word_boundary::WordBoundaryType;

    #[test]
    fn test_insert() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.insert_at_carets("hel");
        b.insert_at_carets("lo");
        assert_eq!("hello", b.content_to_string());
    }

    #[test]
    fn test_insert_at_selection() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.insert_at_carets("hello");
        b.regions.set_primary_caret(Region::sticky(1, 3));
        b.insert_at_carets("X");
        assert_eq!("hXlo", b.content_to_string());
    }

    #[test]
    fn test_delete_forwards() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("a".to_string());
        b.delete_at_carets(Trajectory::Forwards);
        assert_eq!("", b.content_to_string());
    }

    #[test]
    fn test_delete_backwards() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("a".to_string());
        b.regions.set_primary_caret(Region::sticky_cursor(1));
        b.delete_at_carets(Trajectory::Backwards);
        assert_eq!("", b.content_to_string());
    }

    /// For now, `delete_backwards_at_carets` collapses selections into cursors,
    /// and then backspaces as usual. Not sure if this is the behavior we want...
    #[test]
    fn test_delete_selection() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("hello".to_string());
        b.regions.set_primary_caret(Region::sticky(1, 3));
        b.delete_at_carets(Trajectory::Backwards);
        assert_eq!("ello", b.content_to_string());
    }

    #[test]
    fn test_delete_empty() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.delete_at_carets(Trajectory::Backwards);
        assert_eq!("", b.content_to_string());
    }

    #[test]
    fn test_move_next_word_boundary() {
        test_util::setup_test();
        let t = Rope::from("hello hello hello");
        let vp = Viewport::new_ginormeous();
        let motion_start = Motion::NextWordBoundary(WordBoundaryType::Start);
        let motion_end = Motion::NextWordBoundary(WordBoundaryType::End);
        assert_eq!(
            5,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(1), false, motion_end).head
        );
        assert_eq!(
            6,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(1), false, motion_start).head
        );
        assert_eq!(
            12,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(6), false, motion_start).head,
            "Next word boundary should move you, even when starting on a word bounadry",
        );
        assert_eq!(
            17,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(13), false, motion_end).head,
            "End of the string should be seen as a boundary when moving forwards",
        );
    }

    #[test]
    fn test_move_previous_word_boundary() {
        test_util::setup_test();
        let t = Rope::from("hello hello hello");
        let vp = Viewport::new_ginormeous();
        let motion_start = Motion::PrevWordBoundary(WordBoundaryType::Start);
        let motion_end = Motion::PrevWordBoundary(WordBoundaryType::End);
        assert_eq!(
            0,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(3), false, motion_start).head,
            "Start of the string should be seen as a boundary when moving backwards",
        );
        assert_eq!(
            0,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(3), false, motion_start).head,
            "Start of the string should be seen as a boundary when moving backwards",
        );
        assert_eq!(
            5,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(8), false, motion_end).head
        );
        assert_eq!(
            6,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(8), false, motion_start).head
        );
        assert_eq!(
            0,
            apply_motion_to_region(&t, &vp, Region::sticky_cursor(6), false, motion_start).head
        );
    }

    #[test]
    fn test_move_caret_selecting() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("hello, world".to_string());
        let vp = Viewport::new_ginormeous();
        b.apply_buffer_op(&vp, BufferOp::Selection(Motion::Right));
        b.apply_buffer_op(&vp, BufferOp::Selection(Motion::Right));
        assert_eq!((0..2), b.regions.carets().first().range());
    }

    #[test]
    fn test_move_caret_remembers_column() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("hello\nxx\nworld".to_string());
        b.regions.set_primary_caret(Region::sticky_cursor(3));
        let vp = Viewport::new_ginormeous();
        b.apply_buffer_op(&vp, BufferOp::Move(Motion::Down));
        b.apply_buffer_op(&vp, BufferOp::Move(Motion::Down));
        assert_eq!(12, b.regions.carets().first().head);
    }

    #[test]
    fn test_move_caret_forgets_column_after_horiz_movement() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("hello\nxxx\nworld".to_string());
        b.regions.set_primary_caret(Region::sticky_cursor(12));
        let vp = Viewport::new_ginormeous();
        b.apply_buffer_op(&vp, BufferOp::Move(Motion::Up));
        b.apply_buffer_op(&vp, BufferOp::Move(Motion::Left));
        b.apply_buffer_op(&vp, BufferOp::Move(Motion::Up));
        assert_eq!(1, b.regions.carets().first().head);
    }

    #[test]
    fn test_move_collapses_selection() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("hello, world".to_string());
        let vp = Viewport::new_ginormeous();
        b.apply_buffer_op(&vp, BufferOp::Selection(Motion::Right));
        b.apply_buffer_op(&vp, BufferOp::Selection(Motion::Right));
        assert_eq!((0..2), b.regions.carets().first().range());
        b.apply_buffer_op(&vp, BufferOp::Move(Motion::Right));
        assert_eq!((3..3), b.regions.carets().first().range());
    }

    #[test]
    fn test_move_caret_empty() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        // An empty file doesn't allow much movement...
        // Let's hope we don't break the walls
        b.move_carets(&vp, Motion::Left);
        b.move_carets(&vp, Motion::Right);
        b.move_carets(&vp, Motion::Down);
        b.move_carets(&vp, Motion::Up);
        b.move_carets(&vp, Motion::StartOfLine);
        b.move_carets(&vp, Motion::EndOfLine);
        b.move_carets(&vp, Motion::TopOfViewport);
        b.move_carets(&vp, Motion::BottomOfViewport);
    }

    #[test]
    fn test_move_caret_edges() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        // Let's just spam moving into the walls and see if it breaks
        b.insert_at_carets("hi\nho");
        b.move_carets(&vp, Motion::Down);
        b.move_carets(&vp, Motion::Down);
        assert_eq!(b.text.len(), b.regions.carets().first().head);
        b.move_carets(&vp, Motion::Right);
        b.move_carets(&vp, Motion::Right);
        assert_eq!(b.text.len(), b.regions.carets().first().head);
        b.move_carets(&vp, Motion::Up);
        b.move_carets(&vp, Motion::Up);
        b.move_carets(&vp, Motion::Up);
        assert_eq!(2, b.regions.carets().first().head);
        b.move_carets(&vp, Motion::Left);
        b.move_carets(&vp, Motion::Left);
        b.move_carets(&vp, Motion::Left);
        assert_eq!(0, b.regions.carets().first().head);
    }

    #[test]
    fn test_move_caret_down_into_shorter_line() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("hello\nX".to_string());
        b.regions.set_primary_caret(Region::sticky_cursor(5));
        let vp = Viewport::new_ginormeous();
        b.move_carets(&vp, Motion::Down);
        assert_eq!(1, b.all_caret_positions().first().line);
        assert_eq!(1, b.all_caret_positions().first().col);
    }

    #[test]
    fn test_move_caret_up_into_shorter_line() {
        test_util::setup_test();
        let mut b = Buffer::new_from_string("X\nhello".to_string());
        b.regions.set_primary_caret(Region::sticky_cursor(5));
        let vp = Viewport::new_ginormeous();
        b.move_carets(&vp, Motion::Up);
        assert_eq!(0, b.all_caret_positions().first().line);
        assert_eq!(1, b.all_caret_positions().first().col);
    }

    #[test]
    fn test_highlevel_movement_line_ends() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        b.insert_at_carets("hello");
        b.move_carets(&vp, Motion::Left);
        b.move_carets(&vp, Motion::Left);
        assert_eq!(3, b.regions.carets().first().head);
        b.move_carets(&vp, Motion::EndOfLine);
        assert_eq!(5, b.regions.carets().first().head);
        b.move_carets(&vp, Motion::StartOfLine);
        assert_eq!(0, b.regions.carets().first().head);
    }

    #[test]
    fn test_highlevel_movement_viewport() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let mut vp = Viewport {
            first_line: 1,
            height: 2,
        };
        b.insert_at_carets("0000\n1111\n2222\n3333\n4444");
        b.move_carets(&vp, Motion::Up);
        b.move_carets(&vp, Motion::Up);
        assert_eq!(2, b.all_caret_positions().first().line);
        b.move_carets(&vp, Motion::TopOfViewport);
        assert_eq!(1, b.all_caret_positions().first().line);
        b.move_carets(&vp, Motion::BottomOfViewport);
        assert_eq!(3, b.all_caret_positions().first().line);

        // verify we don't die if the bottom of the viewport is below the last line
        vp.height = 100;
        b.move_carets(&vp, Motion::BottomOfViewport);
        assert_eq!(4, b.all_caret_positions().first().line);
    }

    #[test]
    fn test_undo_then_insert() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.insert_at_carets("hey");
        b.undo();
        assert_eq!("", b.content_to_string());
        assert_eq!(
            0,
            b.all_caret_positions().first().to_offset_snapping(&b.text)
        );
        b.insert_at_carets("hello");
        assert_eq!("hello", b.content_to_string());
    }

    #[test]
    fn test_undo_caret_stays_when_before_affected_text() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        b.insert_at_carets("heyy");
        b.delete_at_carets(Trajectory::Backwards);
        b.insert_at_carets("\nho");
        b.move_carets(&vp, Motion::Up);
        b.undo();
        assert_eq!(
            &Position { line: 0, col: 2 },
            b.all_caret_positions().first()
        );
    }

    #[test]
    fn test_undo_caret_moves_when_after_affected_text() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.insert_at_carets("heyy");
        b.delete_at_carets(Trajectory::Backwards);
        b.insert_at_carets("ho");
        b.undo();
        assert_eq!(3, b.all_caret_positions().first().col);
    }

    #[test]
    fn test_undo_redo() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.undo();
        assert_eq!("", b.content_to_string());
        b.insert_at_carets("hey");
        b.delete_at_carets(Trajectory::Backwards);
        b.insert_at_carets("llo");
        b.insert_at_carets(" world");
        assert_eq!("hello world", b.content_to_string());
        b.undo();
        assert_eq!("he", b.content_to_string());
        b.undo();
        assert_eq!("hey", b.content_to_string());
        b.undo();
        assert_eq!("", b.content_to_string());
        b.undo();
        assert_eq!("", b.content_to_string());

        b.redo();
        assert_eq!("hey", b.content_to_string());
        b.undo();
        assert_eq!("", b.content_to_string());
        b.redo();
        assert_eq!("hey", b.content_to_string());
        b.redo();
        assert_eq!("he", b.content_to_string());
        b.redo();
        assert_eq!("hello world", b.content_to_string());
    }

    #[test]
    fn test_undo_edit_redo() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.undo();
        assert_eq!("", b.content_to_string());
        b.insert_at_carets("hey");
        b.delete_at_carets(Trajectory::Backwards);
        b.insert_at_carets("llo");
        assert_eq!("hello", b.content_to_string());
        b.undo();
        b.insert_at_carets("yho");
        assert_eq!("heyho", b.content_to_string());
        b.redo();
        assert_eq!("heyho", b.content_to_string());
    }
}
