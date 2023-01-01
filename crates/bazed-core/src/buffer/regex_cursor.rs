use std::ops::Range;

use hotsauce::Regex;
use xi_rope::Rope;

use crate::word_boundary::iter_rope_chunks_reverse;

pub(crate) struct RegexCursor<'a, 'b> {
    text: &'a Rope,
    pos: usize,
    re: &'b Regex,
}

impl<'a, 'b> RegexCursor<'a, 'b> {
    pub(crate) fn new(text: &'a Rope, pos: usize, re: &'b Regex) -> Self {
        RegexCursor { text, re, pos }
    }

    pub(crate) fn next_match(&mut self) -> Option<Range<usize>> {
        let iter = self.text.iter_chunks(self.pos..).flat_map(|x| x.bytes());
        let mut matches = self.re.matches(iter);
        let m = matches.next()?;
        let actual_range = (self.pos + m.start)..(self.pos + m.end);
        self.pos = actual_range.end;
        Some(actual_range)
    }

    pub(crate) fn prev_match(&mut self) -> Option<Range<usize>> {
        let iter = iter_rope_chunks_reverse(self.text, ..self.pos).flat_map(|x| x.bytes().rev());
        let mut matches = self.re.rmatches(iter);
        let m = matches.next()?;
        let actual_range = (self.pos - m.end)..(self.pos - m.start);
        self.pos = actual_range.start;
        Some(actual_range)
    }
}

#[cfg(test)]
mod test {
    use hotsauce::Regex;
    use xi_rope::Rope;

    use super::RegexCursor;

    #[test]
    fn test_next_match() {
        let r = Rope::from("foo bar baz bar baz");
        let re = Regex::new("bar").unwrap();
        let mut c = RegexCursor::new(&r, 0, &re);
        assert_eq!(Some(4..7), c.next_match());
        assert_eq!(Some(12..15), c.next_match());
        assert_eq!(None, c.next_match());
    }

    #[test]
    fn test_prev_match() {
        let r = Rope::from("foo bar baz bar baz");
        let re = Regex::new("bar").unwrap();
        let mut c = RegexCursor::new(&r, 19, &re);
        assert_eq!(Some(12..15), c.prev_match());
        assert_eq!(Some(4..7), c.prev_match());
        assert_eq!(None, c.prev_match());
    }

    #[test]
    fn test_prev_match_edge() {
        let r = Rope::from("foo x foo");
        let re = Regex::new("foo").unwrap();
        let mut c = RegexCursor::new(&r, 9, &re);
        assert_eq!(Some(6..9), c.prev_match());
        assert_eq!(Some(0..3), c.prev_match());
        assert_eq!(None, c.prev_match());
    }
}
