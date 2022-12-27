use unicode_general_category::GeneralCategory;
use xi_rope::Rope;

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
/// This iterator will always emit an `End`-boundary at the end of the text.
/// Thus, this is guaranteed to always at least yield one value.
pub(crate) struct WordBoundaries<I> {
    iter: I,
    prev: Option<char>,
    /// The index of the cursor. `prev` is to the left of this (effectively at `current_offset - 1`)
    current_offset: usize,
    /// true if the iterator has been exhausted and final `End` has already been yielded
    done: bool,
}
impl<I: Iterator<Item = char>> WordBoundaries<I> {
    /// Offsets are added to the initial offset
    pub(crate) fn from_iter<It: IntoIterator<IntoIter = I>>(
        iter: It,
        initial_offset: usize,
    ) -> Self {
        Self {
            iter: iter.into_iter(),
            prev: None,
            done: false,
            current_offset: initial_offset,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for WordBoundaries<I> {
    type Item = (usize, WordBoundaryType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        loop {
            self.current_offset += 1;
            let Some(cur) = self.iter.next() else {
                self.done = true;
                return Some((self.current_offset - 1, WordBoundaryType::End));
            };
            let prev = self.prev;
            self.prev = Some(cur);
            if let Some(prev) = prev {
                if let Some(boundary) = WordBoundaryType::between(prev, cur) {
                    return Some((self.current_offset - 1, boundary));
                }
            }
        }
    }
}

pub(crate) fn find_word_boundaries(
    rope: &Rope,
    start_at: usize,
) -> impl Iterator<Item = (usize, WordBoundaryType)> + '_ {
    WordBoundaries::from_iter(
        rope.iter_chunks(start_at..).flat_map(|c| c.chars()),
        start_at,
    )
}

#[cfg(test)]
mod test {
    use super::WordBoundaries;
    use crate::{test_util, word_boundary::WordBoundaryType};

    fn boundaries(s: &str) -> Vec<(usize, WordBoundaryType)> {
        WordBoundaries::from_iter(s.chars(), 0).collect()
    }

    #[test]
    fn test_word_boundaries() {
        test_util::setup_test();
        use WordBoundaryType::*;
        let actual = boundaries("foo foo...");
        assert_eq!(vec![(3, End), (4, Start), (7, Both), (10, End)], actual);
        let actual = boundaries(" foo ");
        assert_eq!(vec![(1, Start), (4, End), (5, End)], actual);

        // TODO we should have configurable word separators to allow `_` to not break words if the users wants that
        let actual = boundaries("foo_bar");
        assert_eq!(vec![(3, Both), (4, Both), (7, End)], actual);
        let actual = boundaries("foo___");
        assert_eq!(vec![(3, Both), (6, End)], actual);
    }
}
