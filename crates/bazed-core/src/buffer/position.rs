use xi_rope::Rope;

/// Position in a [crate::buffer::Buffer] by it's line and col.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    /// Get the position of a given offset in a text.
    /// Will return `None` if the offset is outside the bounds of the text
    /// If `offset == text.len()`, will return position of `text.len()`.
    pub fn from_offset(text: &Rope, offset: usize) -> Option<Self> {
        if offset > text.len() {
            None
        } else {
            let line = text.line_of_offset(offset);
            let col = offset - text.offset_of_line(line);
            Some(Position { line, col })
        }
    }

    /// Get the position of a given offset in a text.
    /// If `offset >= text.len()`, will return position of `text.len()`.
    pub fn from_offset_snapping(text: &Rope, offset: usize) -> Self {
        let offset = offset.min(text.len());
        let line = text.line_of_offset(offset);
        let col = offset - text.offset_of_line(line);
        Position { line, col }
    }

    /// Turn a position into an offset at that point.
    /// Return `None` if there is no character at that point.
    /// When the position points to the exact end of the text, will succeed to yield text.len(),
    /// as that is a valid character offset, even if it's not a valid character _index_.
    pub fn to_offset(self, text: &Rope) -> Option<usize> {
        let last_line = text.line_of_offset(text.len());
        if self.line > last_line {
            return None;
        }
        let line_offset = text.offset_of_line(self.line);
        let naive_offset = line_offset + self.col;
        if naive_offset > text.len() {
            return None;
        }

        // This is very carefully ordered to avoid panics in offset_of_line and prev_grapheme_offset

        // Make sure that naive_offset turns out to be in the line it should
        if self.line == last_line || naive_offset < text.offset_of_line(self.line + 1) {
            if naive_offset == text.len() {
                Some(naive_offset)
            } else {
                // Snap the naive offset to the start of the grapheme it's in
                // If there is no previous grapheme, we're probably at the first -> default to 0
                Some(text.prev_grapheme_offset(naive_offset + 1).unwrap_or(0))
            }
        } else {
            // The naive offset we calculated turned out to be behind the next newline
            // thus position had a valid line, but the column number was too high
            None
        }
    }

    /// Turn a position into an offset at that point,
    /// snapping to the end of the line if the column is further than the line is long
    /// and snapping to the last line if the position in a line that doesn't exist.
    pub fn to_offset_snapping(mut self, text: &Rope) -> usize {
        let last_line = text.line_of_offset(text.len());
        // snap to the last line
        self.line = self.line.min(last_line);
        let line_offset = text.offset_of_line(self.line);

        let naive_offset = line_offset + self.col;
        let naive_offset = if naive_offset >= text.len() {
            // if we're at the end of the text, snap to end of text here
            text.len()
        } else {
            text.prev_grapheme_offset(naive_offset + 1).unwrap_or(0)
        };

        // restrict naive_offset to at max be the end of the given line
        if self.line == last_line {
            // we know that the offset is in bounds of the text,
            // as snapped it to the text length before
            naive_offset
        } else {
            let next_line_offset = text.offset_of_line(self.line + 1);
            if naive_offset < next_line_offset {
                naive_offset
            } else {
                text.prev_grapheme_offset(next_line_offset)
                    .unwrap_or(naive_offset)
            }
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
    use xi_rope::Rope;

    use super::*;
    use crate::test_util;

    #[test]
    fn test_to_offset_valid() {
        // Test that both to_offset and to_offset_snapping work for all valid positions
        // and yield the same value
        let test_to_offset = |t, expected, p: Position| {
            assert_eq!(Some(expected), p.to_offset(t), "to_offset non-snapping");
            assert_eq!(expected, p.to_offset_snapping(t), "to_offset_snapping");
        };

        test_util::setup_test();
        let t = Rope::from("");
        test_to_offset(&t, 0, Position::new(0, 0));

        let t = Rope::from("hello");
        test_to_offset(&t, 5, Position::new(0, 5));

        let t = Rope::from("foo\nx\nbar\n\n");
        test_to_offset(&t, 0, Position::new(0, 0));
        test_to_offset(&t, 3, Position::new(0, 3));
        test_to_offset(&t, 4, Position::new(1, 0));
        test_to_offset(&t, 5, Position::new(1, 1));
        test_to_offset(&t, 6, Position::new(2, 0));
        test_to_offset(&t, 10, Position::new(3, 0));
        test_to_offset(&t, 11, Position::new(4, 0));
    }

    #[test]
    fn test_to_offset_snapping() {
        test_util::setup_test();
        let t = Rope::from("");
        assert_eq!(0, Position::new(10, 0).to_offset_snapping(&t));
        assert_eq!(0, Position::new(10, 10).to_offset_snapping(&t));

        let t = Rope::from("hello");
        assert_eq!(5, Position::new(0, 10).to_offset_snapping(&t));
        assert_eq!(2, Position::new(10, 2).to_offset_snapping(&t));

        let t = Rope::from("foo\n\nbar");
        assert_eq!(3, Position::new(0, 10).to_offset_snapping(&t));
        assert_eq!(4, Position::new(1, 10).to_offset_snapping(&t));
        assert_eq!(8, Position::new(2, 10).to_offset_snapping(&t));
        assert_eq!(8, Position::new(10, 10).to_offset_snapping(&t));
    }

    #[test]
    fn test_from_offset_valid() {
        test_util::setup_test();

        let t = Rope::from("hello");
        assert_eq!(Some(Position::new(0, 5)), Position::from_offset(&t, 5));

        let t = Rope::from("foo\nx\nbar\n\n");
        assert_eq!(Some(Position::new(0, 0)), Position::from_offset(&t, 0));
        assert_eq!(Some(Position::new(0, 3)), Position::from_offset(&t, 3));
        assert_eq!(Some(Position::new(1, 0)), Position::from_offset(&t, 4));
        assert_eq!(Some(Position::new(1, 1)), Position::from_offset(&t, 5));
        assert_eq!(Some(Position::new(2, 0)), Position::from_offset(&t, 6));
        assert_eq!(Some(Position::new(3, 0)), Position::from_offset(&t, 10));
        assert_eq!(Some(Position::new(4, 0)), Position::from_offset(&t, 11));
    }

    #[test]
    fn test_to_offset_invalid() {
        test_util::setup_test();

        let t = Rope::from("hello");
        assert_eq!(None, Position::new(0, 10).to_offset(&t));

        let t = Rope::from("foo\nx\nbar\n\n");
        assert_eq!(Some(0), Position::new(0, 0).to_offset(&t));
        assert_eq!(Some(3), Position::new(0, 3).to_offset(&t));
        assert_eq!(Some(4), Position::new(1, 0).to_offset(&t));
        assert_eq!(Some(5), Position::new(1, 1).to_offset(&t));
        assert_eq!(Some(6), Position::new(2, 0).to_offset(&t));
        assert_eq!(Some(10), Position::new(3, 0).to_offset(&t));
        assert_eq!(Some(11), Position::new(4, 0).to_offset(&t));
    }

    #[test]
    fn test_from_offset_invalid() {
        test_util::setup_test();

        let t = Rope::from("");
        assert_eq!(None, Position::from_offset(&t, 1));

        let t = Rope::from("hello");
        assert_eq!(None, Position::from_offset(&t, 6));
        assert_eq!(None, Position::from_offset(&t, 6000));
    }
}
