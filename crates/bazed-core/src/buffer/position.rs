use xi_rope::Rope;

/// Position in a [bazed_core::buffer::Buffer] by it's line and col.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}

impl Position {
    pub fn from_offset(text: &Rope, offset: usize) -> Self {
        let line = text.line_of_offset(offset);
        let col = offset - text.offset_of_line(line);
        Position { line, col }
    }

    /// Turn a position into an offset at that point,
    /// snapping to the end of the line if the cursors column is further than the line is long.
    ///
    /// ```rust
    /// use bazed_core::buffer::Position;
    /// use xi_rope::Rope;
    ///
    /// let t = Rope::from("1234\n12\n");
    /// assert_eq!(0, Position { line: 0, col: 0 }.to_offset(&t));
    /// assert_eq!(3, Position { line: 0, col: 3 }.to_offset(&t));
    /// assert_eq!(5, Position { line: 1, col: 0 }.to_offset(&t));
    /// assert_eq!(7, Position { line: 1, col: 10 }.to_offset(&t));
    /// assert_eq!(8, Position { line: 2, col: 0 }.to_offset(&t));
    /// assert_eq!(8, Position { line: 2, col: 10 }.to_offset(&t));
    /// ```
    pub fn to_offset(self, text: &Rope) -> usize {
        let line_offset = text.offset_of_line(self.line);

        let naive_offset = if line_offset + self.col >= text.len() {
            text.len()
        } else {
            text.prev_grapheme_offset(line_offset + self.col + 1)
                .unwrap_or(0)
        };

        // restrict naive_offset to at max be the end of the given line
        let last_line = text.line_of_offset(text.len());
        if self.line == last_line {
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
