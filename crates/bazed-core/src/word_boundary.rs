//! Traverse a text, looking for word boundaries.

use unicode_general_category::GeneralCategory;
use xi_rope::{interval::IntervalBounds, Cursor, Interval, Rope, RopeInfo};

/// Search forwards for any word boundaries in a rope, starting at a given offset.
/// Note that the location at the offset itself is not considered.
///
/// Will always yield a [WordBoundaryType::Both] at the end of the text.
pub(crate) fn find_word_boundaries(
    rope: &Rope,
    start_at: usize,
) -> impl Iterator<Item = (usize, WordBoundaryType)> + '_ {
    WordBoundaries::from_iter(false, rope.iter_chunks(start_at..).flat_map(|c| c.chars()))
        .map(move |(offset, t)| (offset + start_at, t))
        .chain(std::iter::once((rope.len(), WordBoundaryType::Both)))
}

/// Search backwards for any word boundaries in a rope, starting at a given offset.
/// Note that the location at the offset itself is not considered.
///
/// Will always yield a [WordBoundaryType::Both] at the start of the text.
pub(crate) fn find_word_boundaries_backwards(
    rope: &Rope,
    start_at: usize,
) -> impl Iterator<Item = (usize, WordBoundaryType)> + '_ {
    WordBoundaries::from_iter(
        true,
        iter_rope_chunks_reverse(rope, ..start_at).flat_map(|c| c.chars().rev()),
    )
    .map(move |(offset, t)| (start_at.saturating_sub(offset), t))
    .chain(std::iter::once((0, WordBoundaryType::Both)))
}

/// Type of a word-boundary.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub(crate) enum WordBoundaryType {
    /// Indicates the start of a "word"
    Start,
    /// Indicates the end of a "word"
    End,
    /// Indicates that the position is both a start and an end,
    /// i.e. in the case of `a__`, at offset 1, as the underscores
    /// are also grouped into one word.
    Both,
}

impl WordBoundaryType {
    pub(crate) fn between_types(a: CharCategory, b: CharCategory) -> Option<WordBoundaryType> {
        use CharCategory::*;
        use WordBoundaryType::*;
        match (a, b) {
            (Word, Word)
            | (Punctuation, Punctuation)
            | (Whitespace, Whitespace)
            | (Other, _)
            | (_, Other) => None,

            (Word, Punctuation) | (Punctuation, Word) => Some(Both),
            (Whitespace, Word) => Some(Start),
            (Word, Whitespace) => Some(End),
            (Whitespace, Punctuation) => Some(Start),
            (Punctuation, Whitespace) => Some(End),
        }
    }
    pub(crate) fn between(a: char, b: char) -> Option<WordBoundaryType> {
        Self::between_types(CharCategory::of_char(a), CharCategory::of_char(b))
    }

    /// Compare two word boundaries, checking if they match.
    /// If either boundary is `Other` this will always match
    pub(crate) fn matches(&self, other: &Self) -> bool {
        self == &Self::Both || other == &Self::Both || self == other
    }
}

/// Category of character, loosely derived from the unicode general category defined
/// in [Unicode Standard Annex #44, Section 5.7.1](http://www.unicode.org/reports/tr44/#General_Category_Values)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum CharCategory {
    /// Any whitespace or lineseparator character
    Whitespace,
    /// Any word character.
    /// In the future, it should be possible to configure if characters like `_` or `-` count as word-characters or not.
    Word,
    /// Any punctuation character.
    Punctuation,
    /// Any character that doesn't fit any other category.
    /// This also includes format control characters, UTF-16 surrogate code points.
    Other,
}

impl CharCategory {
    fn of_char(c: char) -> Self {
        if c.is_whitespace() {
            return Self::Whitespace;
        }

        use GeneralCategory::*;
        match unicode_general_category::get_general_category(c) {
            OtherPunctuation | OpenPunctuation | ClosePunctuation | InitialPunctuation
            | FinalPunctuation | ConnectorPunctuation | DashPunctuation | MathSymbol
            | CurrencySymbol | ModifierSymbol => Self::Punctuation,

            LineSeparator | ParagraphSeparator | SpaceSeparator => Self::Whitespace,

            LowercaseLetter | TitlecaseLetter | UppercaseLetter | ModifierLetter | OtherLetter
            | OtherNumber | LetterNumber | DecimalNumber | OtherSymbol => Self::Word,

            Control | Format | Surrogate | Unassigned | PrivateUse | SpacingMark
            | NonspacingMark | EnclosingMark => Self::Other,
        }
    }
}

/// Boundaries are always between chars:
/// in `"foo bar"`, there is an `End`-boundary at index 3 (`"foo| bar"`),
/// as well as a `Start`-boundary at index 4 (`"foo |bar"`).
/// Thus: An `End`-boundary at index N really means the character at index (N-1) is in a word, and at index N is not.
///
/// This iterator will not emit an `End`-boundary at the end of the text.
pub(crate) struct WordBoundaries<I> {
    iter: I,
    prev: Option<char>,
    /// The index of the cursor. `prev` is to the left of this (effectively at `current_offset - 1`)
    current_offset: usize,
    /// when true, the previous character and current character will be swapped in boundary checks
    reversing: bool,
}
impl<I: Iterator<Item = char>> WordBoundaries<I> {
    pub(crate) fn from_iter<It: IntoIterator<IntoIter = I>>(reversing: bool, iter: It) -> Self {
        Self {
            iter: iter.into_iter(),
            prev: None,
            current_offset: 0,
            reversing,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for WordBoundaries<I> {
    type Item = (usize, WordBoundaryType);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.current_offset += 1;
            let cur = self.iter.next()?;
            let prev = self.prev;
            self.prev = Some(cur);
            if let Some(prev) = prev {
                let boundary = if self.reversing {
                    WordBoundaryType::between(cur, prev)
                } else {
                    WordBoundaryType::between(prev, cur)
                };
                if let Some(boundary) = boundary {
                    return Some((self.current_offset - 1, boundary));
                }
            }
        }
    }
}

/// Returns an iterator that reverses over chunks of the rope.
/// **Note** that the chunks contents are still ordered left to right.
pub(crate) fn iter_rope_chunks_reverse<T: IntervalBounds>(
    rope: &Rope,
    range: T,
) -> ReverseChunkIter {
    let Interval { start, end } = range.into_interval(rope.len());
    ReverseChunkIter {
        cursor: Cursor::new(rope, end),
        cursor_at_end: true,
        min: start,
    }
}

/// More or less a copy of [xi_rope::rope::ChunkIter], but traversing the ropes chunks in reverse order.
/// **Note** that the chunks contents are still ordered left to right.
pub(crate) struct ReverseChunkIter<'a> {
    cursor: Cursor<'a, RopeInfo>,
    cursor_at_end: bool,
    min: usize,
}

impl<'a> Iterator for ReverseChunkIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.cursor.pos() < self.min {
            return None;
        }
        let (leaf, offset_within_leaf) = self.cursor.get_leaf()?;
        let start_offset_of_leaf = self.cursor.pos() - offset_within_leaf;
        let start_pos = self.min.saturating_sub(start_offset_of_leaf);
        self.cursor.prev_leaf();
        if self.cursor_at_end {
            self.cursor_at_end = false;
            Some(&leaf[start_pos..offset_within_leaf])
        } else {
            Some(&leaf[start_pos..])
        }
    }
}

#[cfg(test)]
mod test {
    use xi_rope::Rope;

    use super::{iter_rope_chunks_reverse, WordBoundaries};
    use crate::{
        test_util,
        word_boundary::{find_word_boundaries, find_word_boundaries_backwards, WordBoundaryType},
    };

    #[test]
    fn test_word_boundaries() {
        test_util::setup_test();
        use WordBoundaryType::*;
        fn boundaries(s: &str) -> Vec<(usize, WordBoundaryType)> {
            WordBoundaries::from_iter(false, s.chars()).collect()
        }
        let actual = boundaries("foo foo...");
        assert_eq!(vec![(3, End), (4, Start), (7, Both)], actual);
        let actual = boundaries(" foo ");
        assert_eq!(vec![(1, Start), (4, End)], actual);

        // TODO we should have configurable word separators to allow `_` to not break words if the users wants that
        let actual = boundaries("foo_bar");
        assert_eq!(vec![(3, Both), (4, Both)], actual);
        let actual = boundaries("foo___");
        assert_eq!(vec![(3, Both)], actual);
    }

    #[test]
    fn test_word_boundaries_in_rope() {
        test_util::setup_test();
        use WordBoundaryType::*;
        assert_eq!(
            vec![(4, End), (5, Start), (8, Both), (11, Both)],
            find_word_boundaries(&Rope::from(" foo foo..."), 2).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn test_word_boundaries_in_rope_reverse() {
        test_util::setup_test();
        use WordBoundaryType::*;
        assert_eq!(
            vec![(5, Start), (4, End), (1, Start), (0, Both)],
            find_word_boundaries_backwards(&Rope::from(" foo foo..."), 7).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn test_reverse_chunks_iter() {
        test_util::setup_test();
        assert_eq!(
            "0123",
            iter_rope_chunks_reverse(&"0123".into(), ..).collect::<String>()
        );
        assert_eq!(
            "01",
            iter_rope_chunks_reverse(&"0123".into(), ..2).collect::<String>()
        );
        assert_eq!(
            "23",
            iter_rope_chunks_reverse(&"0123".into(), 2..).collect::<String>()
        );
        assert_eq!(
            "12",
            iter_rope_chunks_reverse(&"0123".into(), 1..3).collect::<String>()
        );
        assert_eq!(
            "0123",
            iter_rope_chunks_reverse(&"0123".into(), 0..4).collect::<String>()
        );
    }
}
