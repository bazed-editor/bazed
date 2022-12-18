use std::collections::{BTreeSet, HashMap};

use nonempty::{nonempty, NonEmpty};
use tap::Pipe;
use xi_rope::{engine::Engine, tree::NodeInfo, DeltaBuilder, Rope, RopeDelta, Transformer};

use crate::{
    mark::{Mark, MarkId},
    user_buffer_op::{BufferOp, EditOp, MovementOp},
};

/// Stores all the active marks in a buffer.
///
/// Terminology:
/// - *Mark* refers to any marked position in the buffer
/// - *Caret* refers to marks that represent concrete, user-controlled carets.
///   (i.e.: The places where text gets inserted)
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

    fn carets(&self) -> NonEmpty<Mark> {
        self.carets
            .iter()
            .map(|x| *self.marks.get(x).expect("caret not found in marks"))
            .collect::<Vec<_>>()
            .pipe(NonEmpty::from_vec)
            .unwrap()
    }

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

pub struct Buffer {
    text: Rope,
    engine: Engine,
    undo_group_id: usize,
    marks: BufferMarks,
}

impl Buffer {
    pub fn open_ephemeral() -> Self {
        let rope = Rope::from(String::new());
        Self {
            undo_group_id: 1,
            engine: Engine::new(rope.clone()),
            text: rope,
            marks: BufferMarks::default(),
        }
    }

    pub fn content_to_string(&self) -> String {
        self.engine.get_head().to_string()
    }

    pub fn all_carets(&self) -> NonEmpty<Position> {
        self.marks
            .carets()
            .map(|x| Position::from_offset(&self.text, x.offset))
    }

    #[tracing::instrument(skip(self))]
    fn commit_delta(&mut self, delta: RopeDelta) -> Rope {
        let head_rev = self.engine.get_head_rev_id();
        let undo_group = self.calculate_undo_group();
        //self.last_edit_type = self.this_edit_type;

        self.marks.apply_delta(&delta);
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
        for mark in self.marks.carets() {
            builder.replace(mark, text.clone());
        }
        let delta = builder.build();
        self.commit_delta(delta);
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
        for mark in self.marks.carets_mut() {
            *mark = apply_movement_to_mark(&self.text, *mark, op);
        }
    }
}

fn apply_movement_to_mark(text: &Rope, mark: Mark, op: MovementOp) -> Mark {
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
        let next_line_offset = text.offset_of_line(self.line + 1);
        // TODO does that unwrap_or make sense?
        let naive_offset = text
            .prev_grapheme_offset(line_offset + self.col + 1)
            .unwrap_or(text.len());

        // restrict naive_offset to at max be the end of the given line
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
    #[test]
    fn test_insert() {
        let mut b = Buffer::open_ephemeral();
        b.insert_at_carets("hel");
        b.insert_at_carets("lo");
        assert_eq!("hello", b.content_to_string());
    }

    #[test]
    fn test_backspace() {
        let mut b = Buffer::open_ephemeral();
        b.insert_at_carets("a");
        assert_eq!("a", b.content_to_string());
        b.delete_backward_at_carets();
        assert_eq!("", b.content_to_string());
        b.delete_backward_at_carets();
        assert_eq!("", b.content_to_string());
    }

    #[test]
    fn test_move_caret_into_shorter_line() {
        let mut b = Buffer::open_ephemeral();
        b.insert_at_carets("hi\nworld");
        b.apply_movement_op(MovementOp::Up);
        b.insert_at_carets(",");
        assert_eq!("hi,\nworld", b.content_to_string());
    }
}
