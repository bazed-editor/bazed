use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum KeyInputParseError {
    #[error("Could not parse modifier {0}")]
    InvalidModifier(String),
    #[error("Input was empty")]
    EmptyInput,
}

/// A combination of held [Modifier]s and a [Key].
// TODO figure out normalization: Do we get `Shift+a` or do we get `Key::Char('A')`?
#[derive(PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KeyInput {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

impl KeyInput {
    pub fn shift_held(&self) -> bool {
        self.modifiers.contains(&Modifier::Shift)
    }
    pub fn ctrl_held(&self) -> bool {
        self.modifiers.contains(&Modifier::Ctrl)
    }
    pub fn alt_held(&self) -> bool {
        self.modifiers.contains(&Modifier::Alt)
    }
    pub fn win_held(&self) -> bool {
        self.modifiers.contains(&Modifier::Win)
    }
}

impl std::fmt::Display for KeyInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "<{}-{}>", self.modifiers.iter().join("-"), self.key)
        }
    }
}

impl FromStr for KeyInput {
    type Err = KeyInputParseError;

    // TODO related to normalization: Do we wanna parse `A` as `<S-A>`, `A` or `<S-a>`?
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
            let mut parts = s.split('-').peekable();
            let mut modifiers = Vec::new();
            while let Some(part) = parts.next() {
                if parts.peek().is_some() {
                    modifiers.push(part.parse()?);
                } else {
                    return Ok(Self {
                        modifiers,
                        key: Key(part.to_string()),
                    });
                }
            }
            Err(KeyInputParseError::EmptyInput)
        } else {
            Ok(Self {
                modifiers: Vec::new(),
                key: Key(s.to_string()),
            })
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash, Serialize, Deserialize, derive_more::Display)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    #[display(fmt = "C")]
    Ctrl,
    #[display(fmt = "A")]
    Alt,
    #[display(fmt = "S")]
    Shift,
    #[display(fmt = "M")]
    Win,
}

impl FromStr for Modifier {
    type Err = KeyInputParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match () {
            _ if s.eq_ignore_ascii_case("c") => Ok(Self::Ctrl),
            _ if s.eq_ignore_ascii_case("s") => Ok(Self::Shift),
            _ if s.eq_ignore_ascii_case("m") => Ok(Self::Win),
            _ if s.eq_ignore_ascii_case("a") => Ok(Self::Alt),
            _ => Err(KeyInputParseError::InvalidModifier(s.to_string())),
        }
    }
}

/// Keys are represented by their key attribute values as specified in <https://www.w3.org/TR/uievents-key/>,
/// that being either a [key string](https://www.w3.org/TR/uievents-key/#key-string) or
/// a [named key attribute value](https://www.w3.org/TR/uievents-key/#named-key-attribute-value)
#[derive(PartialEq, Eq, Debug, Hash, Serialize, Deserialize, derive_more::Display)]
#[serde(rename_all = "snake_case")]
pub struct Key(String);

impl Key {
    pub fn is_key_string(&self) -> bool {
        use unicode_canonical_combining_class::{
            get_canonical_combining_class, CanonicalCombiningClass,
        };
        use unicode_general_category::{get_general_category, GeneralCategory};

        fn is_non_control_character(c: char) -> bool {
            get_general_category(c) != GeneralCategory::Control
        }
        fn is_combining_character(c: char) -> bool {
            get_general_category(c) == GeneralCategory::SpacingMark
                || get_canonical_combining_class(c) != CanonicalCombiningClass::NotReordered
        }

        let mut chars = self.0.chars();
        // If there is no character, it's always a valid key string
        let Some(first) = chars.next() else { return true };
        // 0 or 1 non-control characters
        if !is_non_control_character(first) && !is_combining_character(first) {
            return false;
        }
        // followed by 0 or more combining characters
        chars.all(is_combining_character)
    }

    pub fn is_named_key_attribute_value(&self) -> bool {
        !self.is_key_string()
    }

    pub fn as_key_string(&self) -> Option<&str> {
        self.is_key_string().then_some(&self.0)
    }

    pub fn as_named_key_attribute_value(&self) -> Option<&str> {
        self.is_named_key_attribute_value().then_some(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::{Key, KeyInput};
    use crate::input_event::Modifier;

    #[test]
    fn test_parse_key_input() {
        let key_input = |modifiers, key: &str| KeyInput {
            modifiers,
            key: Key(key.to_string()),
        };
        assert_eq!(key_input(vec![], "Backspace"), "Backspace".parse().unwrap());
        assert_eq!(
            key_input(vec![], "Backspace"),
            "<Backspace>".parse().unwrap(),
        );
        assert_eq!(
            key_input(vec![Modifier::Ctrl], "Backspace"),
            "<C-Backspace>".parse().unwrap(),
        );
        assert_eq!(
            key_input(
                vec![Modifier::Ctrl, Modifier::Shift, Modifier::Alt],
                "Backspace"
            ),
            "<C-S-A-Backspace>".parse().unwrap(),
        );
        assert_eq!(key_input(vec![], "C"), "<C>".parse().unwrap());
    }

    #[test]
    fn test_key_string() {
        #[rustfmt::skip]
        let key_string_examples = [
            "a", "A", "b", "B", "å", "é", "ü", "ñ", "@", "%", "$", "*", "0", "1", "2", "あ", "日",
            "中", "一", "二", "三", "ا", "ب", "ة", "ت", "١", "٢", "٣", "а", "б", "в", "г", "±",
            "ʶ", "϶", "൹", "℉",
            "\u{0020}",
            "\u{00a0}",
            "\u{2009}",
            "\u{3000}",
            "\u{00f4}",
            "\u{1e0d}\u{0307}",
        ];
        for key in key_string_examples {
            assert!(
                Key(key.to_string()).is_key_string(),
                "{} should be a key string",
                key
            );
        }
    }

    #[test]
    fn test_named_key_attribute_values() {
        let named_key_attribute_value_examples =
            ["Backspace", "Tab", "Enter", "Escape", "Delete", "F11"];
        for key in named_key_attribute_value_examples {
            assert!(
                Key(key.to_string()).is_named_key_attribute_value(),
                "{} should be a named key attribute value",
                key
            );
        }
    }
}
