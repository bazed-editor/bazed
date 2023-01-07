use std::str::FromStr;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::key_combo::KeyInputParseError;

/// A combination of held [Modifier]s and a [Key].
// TODO figure out normalization: Do we get `Shift+a` or do we get `Key::Char('A')`?
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KeyInput {
    /// Modifiers held down in this event.
    /// TODO make this some sort of set
    pub modifiers: Vec<Modifier>,
    pub key: Key,
    pub code: RawKey,
}

impl std::fmt::Display for KeyInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "<{}-{}>", self.modifiers.iter().join("-"), self.code)
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, derive_more::Display)]
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
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, derive_more::Display)]
#[serde(rename_all = "snake_case")]
pub struct Key(pub String);

impl Key {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

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

/// Raw key value according to <https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_code_values>
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, derive_more::Display)]
#[serde(rename_all = "snake_case")]
pub struct RawKey(pub String);

impl RawKey {
    pub fn alpha(s: &str) -> Self {
        Self(format!("Key{}", s.to_uppercase()))
    }
    pub fn num(s: &str) -> Self {
        Self(format!("Digit{s}"))
    }
    pub fn key(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<&str> for RawKey {
    fn from(s: &str) -> Self {
        if s.len() == 1 {
            let c = s.chars().next().unwrap();
            if c.is_ascii_alphabetic() {
                Self::alpha(s)
            } else if c.is_numeric() {
                Self::num(s)
            } else {
                Self(s.to_string())
            }
        } else {
            RawKey(s.to_string())
        }
    }
}

#[cfg(test)]
mod test {
    use super::Key;

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
