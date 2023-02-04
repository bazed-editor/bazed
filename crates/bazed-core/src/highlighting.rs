use syntect::parsing::{ParseState, ScopeStack, SyntaxSet};
use xi_rope::{
    spans::{Spans, SpansBuilder},
    tree::NodeInfo,
    Interval, Rope,
};

/// Spans of a rope annotated with respective [ScopeStack]s
#[derive(Debug, Default)]
pub(crate) struct Annotations {
    spans: Spans<ScopeStack>,
}

impl Annotations {
    pub(crate) fn spans(&self) -> &Spans<ScopeStack> {
        &self.spans
    }

    pub(crate) fn set(&mut self, spans: Spans<ScopeStack>) {
        self.spans = spans;
    }

    pub(crate) fn apply_delta<T: NodeInfo>(&mut self, delta: &xi_rope::Delta<T>) {
        self.spans.apply_shape(delta);
    }
}

#[derive(Debug, Default)]
pub(crate) struct Parser {
    syntax_set: SyntaxSet,
}

impl Parser {
    pub(crate) fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }

    pub(crate) fn parse(&self, rope: &Rope) -> Spans<ScopeStack> {
        let syntax_reference = self.syntax_set.find_syntax_by_extension("rs").unwrap();
        let mut state = ParseState::new(syntax_reference);
        let mut spans: SpansBuilder<ScopeStack> = SpansBuilder::new(rope.len());
        let mut start_of_line = 0;
        let mut current_scope_stack = ScopeStack::new();
        let mut last_span = Interval::new(0, 0);
        for line in rope.lines_raw(..) {
            let parsed = state.parse_line(&line, &self.syntax_set).unwrap();
            for (offset, op) in parsed.iter().map(|(col, op)| (col + start_of_line, op)) {
                if last_span.end == offset {
                    current_scope_stack.apply(op).unwrap();
                } else {
                    last_span.end = offset;
                    spans.add_span(last_span, current_scope_stack.clone());
                    current_scope_stack.apply(op).unwrap();
                    last_span = Interval::new(offset, offset);
                }
            }
            start_of_line += line.len();
        }
        spans.add_span(last_span, current_scope_stack.clone());

        spans.build()
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use syntect::parsing::ScopeStack;
    use xi_rope::Rope;

    use crate::highlighting::Parser;

    macro_rules! scopes {
        ($($x:literal),*) => { ScopeStack::from_vec(vec![$($x.parse().unwrap()),*]) }
    }

    #[test]
    fn test_parsing_rust() {
        let text = Rope::from("let\nx = 5\n;");
        let parser = Parser::new();
        let expected = vec![
            ((0..3), scopes!["source.rust", "storage.type.rust"]),
            ((3..6), scopes!["source.rust"]),
            ((6..7), scopes!["source.rust", "keyword.operator.rust"]),
            ((7..8), scopes!["source.rust"]),
            ((8..9), scopes![
                "source.rust",
                "constant.numeric.integer.decimal.rust"
            ]),
            ((9..10), scopes!["source.rust"]),
            ((10..11), scopes![
                "source.rust",
                "punctuation.terminator.rust"
            ]),
            ((11..11), scopes!["source.rust"]),
        ];
        let actual = parser.parse(&text);
        let actual = actual
            .iter()
            .map(|(a, b)| ((a.start..a.end), b.clone()))
            .collect::<Vec<_>>();
        assert_eq!(expected, actual);
    }
}
