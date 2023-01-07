//! Input events, as they are received from a frontend.

use serde::{Deserialize, Serialize};

/// A combination of held [Modifier]s and a [Key].
// TODO figure out normalization: Do we get `Shift+a` or do we get `Key::Char('A')`?
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct KeyInput {
    /// Modifiers held down in this event.
    pub modifiers: Modifiers,
    /// The key that was pressed (see [Key])
    pub key: Key,
    /// Raw key code of the pressed key (see [RawKey])
    pub code: RawKey,
}

impl std::fmt::Display for KeyInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.key)
        } else {
            write!(f, "<{}-{}>", self.modifiers, self.code)
        }
    }
}

bitflags::bitflags! {
    /// Set of held modifiers
    #[derive(Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Modifiers: u8 {
        const CTRL = 0b00000001;
        const SHIFT = 0b00000010;
        const ALT = 0b00000100;
        const WIN = 0b00001000;
    }
}

impl Modifiers {
    /// Parse a char into a single modifier
    pub fn from_char(c: char) -> Option<Modifiers> {
        match () {
            _ if c.eq_ignore_ascii_case(&'c') => Some(Modifiers::CTRL),
            _ if c.eq_ignore_ascii_case(&'s') => Some(Modifiers::SHIFT),
            _ if c.eq_ignore_ascii_case(&'a') => Some(Modifiers::ALT),
            _ if c.eq_ignore_ascii_case(&'m') => Some(Modifiers::WIN),
            _ => None,
        }
    }
}

impl IntoIterator for Modifiers {
    type Item = Self;

    type IntoIter = std::vec::IntoIter<Self>;

    fn into_iter(self) -> Self::IntoIter {
        let mut x = Vec::new();
        if self.contains(Modifiers::CTRL) {
            x.push(Modifiers::CTRL)
        }
        if self.contains(Modifiers::SHIFT) {
            x.push(Modifiers::SHIFT)
        }
        if self.contains(Modifiers::ALT) {
            x.push(Modifiers::ALT)
        }
        if self.contains(Modifiers::WIN) {
            x.push(Modifiers::WIN)
        }
        x.into_iter()
    }
}

impl std::fmt::Display for Modifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        let mut helper = move |c: char| {
            if !first {
                write!(f, "-")?;
            }
            first = false;
            write!(f, "{}", c)
        };
        if self.contains(Modifiers::CTRL) {
            helper('C')?;
        }
        if self.contains(Modifiers::SHIFT) {
            helper('S')?;
        }
        if self.contains(Modifiers::ALT) {
            helper('A')?;
        }
        if self.contains(Modifiers::WIN) {
            helper('W')?;
        }
        Ok(())
    }
}

/// Keys are represented by their key attribute values as specified in <https://www.w3.org/TR/uievents-key/>,
/// that being either a [key string](https://www.w3.org/TR/uievents-key/#key-string) or
/// a [named key attribute value](https://www.w3.org/TR/uievents-key/#named-key-attribute-value)
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize, derive_more::Display)]
#[serde(transparent)]
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
#[serde(transparent)]
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
