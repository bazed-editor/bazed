use std::collections::HashMap;

use nonempty::{nonempty, NonEmpty};
use tap::Pipe;
use xi_rope::{engine::Engine, tree::NodeInfo, DeltaBuilder, Rope, RopeDelta, Transformer};

use self::undo_history::UndoHistory;
use crate::{
    mark::{Mark, MarkId},
    user_buffer_op::{EditOp, EditType, MovementOp},
    view::Viewport,
};

mod undo_history;

/// Stores all the active marks in a buffer.
///
/// Terminology:
/// - *Mark* refers to any marked position in the buffer
/// - *Caret* refers to marks that represent concrete, user-controlled carets.
///   (i.e.: The places where text gets inserted)
#[derive(Debug)]
struct BufferMarks {
    marks: HashMap<MarkId, Mark>,
    /// All the active carets. There will always be at least one.
    /// The first element may be considered the "primary" caret,
    /// being the caret that will remain when exiting any sort of multi-caret mode.
    ///
    /// All possible mutating interactions with [BufferMarks] must guarantee
    /// that all ids stored here continue to actually map to a mark.
    carets: NonEmpty<MarkId>,
}

impl Default for BufferMarks {
    fn default() -> Self {
        let primary_caret = Mark::sticky(0);
        let primary_caret_id = MarkId::gen();
        let marks = maplit::hashmap! { primary_caret_id => primary_caret };
        let carets = nonempty![primary_caret_id];
        Self { marks, carets }
    }
}

impl BufferMarks {
    fn apply_transformer<N: NodeInfo>(&mut self, trans: &mut Transformer<N>) {
        for mark in self.marks.values_mut() {
            mark.apply_transformer(trans);
        }
    }

    fn apply_delta(&mut self, delta: &RopeDelta) {
        let mut transformer = xi_rope::Transformer::new(delta);
        self.apply_transformer(&mut transformer);
    }

    /// Return all carets in this buffer
    fn carets(&self) -> NonEmpty<Mark> {
        self.carets
            .iter()
            .map(|x| *self.marks.get(x).expect("caret not found in marks"))
            .collect::<Vec<_>>()
            .pipe(NonEmpty::from_vec)
            .unwrap()
    }

    /// Return an iterator over mutable references to all carets in this buffer
    fn carets_mut(&mut self) -> impl Iterator<Item = &mut Mark> {
        // TODO This is stupid, but iterating over self.carets instead and getting the refs
        // through get_mut doesn't work trivially, as rust can't verify that we won't get multiple
        // mut refs to the same entry as a result of overlapping keys...
        self.marks
            .iter_mut()
            .filter(|(k, _)| self.carets.contains(k))
            .map(|(_, v)| v)
    }
}

#[derive(Debug)]
pub struct Buffer {
    text: Rope,
    engine: Engine,
    marks: BufferMarks,
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
            marks: BufferMarks::default(),
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
        self.marks
            .carets()
            .map(|x| Position::from_offset(&self.text, x.offset))
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

    /// Snap all marks to the closest valid points in the buffer.
    ///
    /// This may be required if an action (such as undo, currently) changes the buffer
    /// without moving the marks accordingly. In the future, this should not be required
    /// as all actions _should_ move all marks properly, either through a coordinate transform
    /// with [xi_rope::Transformer], or, in the case of undo, by remembering where the carets where before.
    ///
    /// **WARNING:** This is very much a temporary solution, as it _will_ cause inconsistent state as soon as we use
    /// marks for more than just caret position. (see https://github.com/bazed-editor/bazed/issues/47)
    fn snap_marks_to_valid_position(&mut self) {
        for mark in self.marks.marks.values_mut() {
            if mark.offset > self.text.len() {
                mark.offset = self.text.len();
            }
        }
    }

    #[tracing::instrument(skip(self), fields(head_rev_id = ?self.engine.get_head_rev_id()))]
    fn commit_delta(&mut self, delta: RopeDelta, edit_type: EditType) -> Rope {
        tracing::debug!("Committing delta");
        self.marks.apply_delta(&delta);

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
        // This is also where xi handles surrounding stuff in parens when something is selected.
        // i.e. when the text "foo" is in the selection, and the chars are "(",
        // then this would turn the text into "(foo)"
        // We don't yet handle this at all, and I'm not sure if we want to.

        let mut builder = DeltaBuilder::new(self.text.len());
        let text: Rope = chars.into();
        for mark in self.marks.carets() {
            builder.replace(mark, text.clone());
        }
        let delta = builder.build();
        self.commit_delta(delta, EditType::Insert);
    }

    fn delete_backward_at_carets(&mut self) {
        let mut builder = DeltaBuilder::new(self.text.len());
        for mark in self.marks.carets() {
            // See xi-editors `offset_for_delete_backwards` function in backward.rs...
            // all I'll say is `#[allow(clippy::cognitive_complexity)]`.
            let delete_start = 1.max(mark.offset) - 1;
            builder.delete(delete_start..mark.offset);
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
                Ok(delta) => self.marks.apply_delta(&delta),
                Err(err) => {
                    tracing::error!("Error generating delta after undo: {err}");
                    self.snap_marks_to_valid_position();
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
                Ok(delta) => self.marks.apply_delta(&delta),
                Err(err) => {
                    tracing::error!("Error generating delta after redo: {err}");
                    self.snap_marks_to_valid_position();
                },
            }
        }
        tracing::trace!(history = ?self.undo_history, "after redo");
    }

    pub(crate) fn apply_edit_op(&mut self, op: EditOp) {
        match op {
            EditOp::Insert(text) => self.insert_at_carets(&text),
            EditOp::Backspace => self.delete_backward_at_carets(),
            EditOp::Undo => self.undo(),
            EditOp::Redo => self.redo(),
        }
    }

    pub(crate) fn apply_movement_op(&mut self, viewport: &Viewport, op: MovementOp) {
        for mark in self.marks.carets_mut() {
            *mark = apply_movement_to_mark(&self.text, viewport, *mark, op);
        }
    }
}

fn apply_movement_to_mark(text: &Rope, vp: &Viewport, mark: Mark, op: MovementOp) -> Mark {
    let offset = match op {
        MovementOp::Left => text
            .prev_grapheme_offset(mark.offset)
            .unwrap_or(mark.offset),
        MovementOp::Right => text
            .next_grapheme_offset(mark.offset)
            .unwrap_or(mark.offset),
        MovementOp::Up => {
            let pos = Position::from_offset(text, mark.offset);
            if pos.line > 0 {
                pos.with_line(pos.line - 1).to_offset(text)
            } else {
                mark.offset
            }
        },
        MovementOp::Down => {
            let pos = Position::from_offset(text, mark.offset);
            let last_line = text.line_of_offset(text.len());
            if pos.line < last_line {
                pos.with_line(pos.line + 1).to_offset(text)
            } else {
                mark.offset
            }
        },
        MovementOp::StartOfLine => {
            let line = text.line_of_offset(mark.offset);
            text.offset_of_line(line)
        },
        MovementOp::EndOfLine => {
            let line = text.line_of_offset(mark.offset);
            let last_line = text.line_of_offset(text.len());
            if line < last_line {
                text.offset_of_line(line + 1)
            } else {
                text.len()
            }
        },
        MovementOp::TopOfViewport => {
            let current_pos = Position::from_offset(text, mark.offset);
            current_pos.with_line(vp.first_line).to_offset(text)
        },
        MovementOp::BottomOfViewport => {
            let current_pos = Position::from_offset(text, mark.offset);
            let last_line = text.line_of_offset(text.len());
            let target_line = vp.last_line().min(last_line);
            current_pos.with_line(target_line).to_offset(text)
        },
    };
    Mark {
        offset,
        kind: mark.kind,
    }
}

/// Position of a [Mark] in a [Buffer] by it's line and col.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    fn from_offset(text: &Rope, offset: usize) -> Self {
        let line = text.line_of_offset(offset);
        let col = offset - text.offset_of_line(line);
        Position { line, col }
    }
    /// Turn a position into an offset at that point,
    /// snapping to the end of the line if the cursors column is further than the line is long.
    fn to_offset(self, text: &Rope) -> usize {
        let line_offset = text.offset_of_line(self.line);
        let naive_offset = if line_offset + self.col >= text.len() {
            text.len()
        } else {
            text.prev_grapheme_offset(line_offset + self.col + 1)
                .unwrap_or(text.len())
        };

        // restrict naive_offset to at max be the end of the given line
        let next_line_offset = text.offset_of_line(self.line + 1);
        if naive_offset >= next_line_offset {
            text.prev_grapheme_offset(next_line_offset)
                .unwrap_or(naive_offset)
        } else {
            naive_offset
        }
    }

    pub fn with_line(self, line: usize) -> Self {
        Self { line, ..self }
    }

    pub fn with_col(self, col: usize) -> Self {
        Self { col, ..self }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util;
    use crate::view::Viewport;

    #[test]
    fn test_insert() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.insert_at_carets("hel");
        b.insert_at_carets("lo");
        assert_eq!("hello", b.content_to_string());
    }

    #[test]
    fn test_backspace() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.insert_at_carets("a");
        assert_eq!("a", b.content_to_string());
        b.delete_backward_at_carets();
        assert_eq!("", b.content_to_string());
    }

    #[test]
    fn test_backspace_empty() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.delete_backward_at_carets();
        assert_eq!("", b.content_to_string());
    }

    #[test]
    fn test_move_caret_empty() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        // An empty file doesn't allow much movement...
        // Let's hope we don't break the walls
        b.apply_movement_op(&vp, MovementOp::Left);
        b.apply_movement_op(&vp, MovementOp::Right);
        b.apply_movement_op(&vp, MovementOp::Down);
        b.apply_movement_op(&vp, MovementOp::Up);
        b.apply_movement_op(&vp, MovementOp::StartOfLine);
        b.apply_movement_op(&vp, MovementOp::EndOfLine);
        b.apply_movement_op(&vp, MovementOp::TopOfViewport);
        b.apply_movement_op(&vp, MovementOp::BottomOfViewport);
    }

    #[test]
    fn test_move_caret_edges() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        // Let's just spam moving into the walls and see if it breaks
        b.insert_at_carets("hi\nho");
        b.apply_movement_op(&vp, MovementOp::Down);
        b.apply_movement_op(&vp, MovementOp::Down);
        assert_eq!(b.text.len(), b.marks.carets().first().offset);
        b.apply_movement_op(&vp, MovementOp::Right);
        b.apply_movement_op(&vp, MovementOp::Right);
        assert_eq!(b.text.len(), b.marks.carets().first().offset);
        b.apply_movement_op(&vp, MovementOp::Up);
        b.apply_movement_op(&vp, MovementOp::Up);
        b.apply_movement_op(&vp, MovementOp::Up);
        assert_eq!(2, b.marks.carets().first().offset);
        b.apply_movement_op(&vp, MovementOp::Left);
        b.apply_movement_op(&vp, MovementOp::Left);
        b.apply_movement_op(&vp, MovementOp::Left);
        assert_eq!(0, b.marks.carets().first().offset);
    }

    #[test]
    fn test_move_caret_into_shorter_line() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        b.insert_at_carets("hi\nworld");
        b.apply_movement_op(&vp, MovementOp::Up);
        b.insert_at_carets(",");
        assert_eq!("hi,\nworld", b.content_to_string());
    }

    #[test]
    fn test_highlevel_movement_line_ends() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        b.insert_at_carets("hello");
        b.apply_movement_op(&vp, MovementOp::Left);
        b.apply_movement_op(&vp, MovementOp::Left);
        assert_eq!(3, b.marks.carets().first().offset);
        b.apply_movement_op(&vp, MovementOp::EndOfLine);
        assert_eq!(5, b.marks.carets().first().offset);
        b.apply_movement_op(&vp, MovementOp::StartOfLine);
        assert_eq!(0, b.marks.carets().first().offset);
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
        b.apply_movement_op(&vp, MovementOp::Up);
        b.apply_movement_op(&vp, MovementOp::Up);
        assert_eq!(2, b.all_caret_positions().first().line);
        b.apply_movement_op(&vp, MovementOp::TopOfViewport);
        assert_eq!(1, b.all_caret_positions().first().line);
        b.apply_movement_op(&vp, MovementOp::BottomOfViewport);
        assert_eq!(3, b.all_caret_positions().first().line);

        // verify we don't die if the bottom of the viewport is below the last line
        vp.height = 100;
        b.apply_movement_op(&vp, MovementOp::BottomOfViewport);
        assert_eq!(4, b.all_caret_positions().first().line);
    }

    #[test]
    fn test_undo_then_insert() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        b.insert_at_carets("hey");
        b.undo();
        assert_eq!("", b.content_to_string());
        assert_eq!(0, b.all_caret_positions().first().to_offset(&b.text));
        b.insert_at_carets("hello");
        assert_eq!("hello", b.content_to_string());
    }

    #[test]
    fn test_undo_caret_stays_when_before_affected_text() {
        test_util::setup_test();
        let mut b = Buffer::new_empty();
        let vp = Viewport::new_ginormeous();
        b.insert_at_carets("heyy");
        b.delete_backward_at_carets();
        b.insert_at_carets("\nho");
        b.apply_movement_op(&vp, MovementOp::Up);
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
        b.delete_backward_at_carets();
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
        b.delete_backward_at_carets();
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
        b.delete_backward_at_carets();
        b.insert_at_carets("llo");
        assert_eq!("hello", b.content_to_string());
        b.undo();
        b.insert_at_carets("yho");
        assert_eq!("heyho", b.content_to_string());
        b.redo();
        assert_eq!("heyho", b.content_to_string());
    }
}
