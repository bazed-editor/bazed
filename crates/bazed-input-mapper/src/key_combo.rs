//! Key combo definitions as they are used in specifying keymaps.

use std::str::FromStr;

use itertools::Itertools;

use crate::input_event::{Key, KeyInput, Modifier, RawKey};

/// Specification of a keypress, either through raw key codes or through key attribute value,
/// designed for use in keymaps
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeySpec {
    /// Key specified as a raw key code (<https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_code_values>)
    Raw(RawKey),
    /// Key specified as a key attribute value (<https://www.w3.org/TR/uievents-key/>)
    Str(Key),
}

impl std::fmt::Display for KeySpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Raw(s) => write!(f, "{}", s.0.replace("Digit", "").replace("Key", "")),
            Self::Str(s) => write!(f, "{}", s.0),
        }
    }
}

/// Specification of a combination of a key and 0 or more modifiers that where held down.
///
/// This differs from [KeyInput] in that it is specified via _either_ a raw key code,
/// _or_ a key attribute value, whereas [KeyInput] provides both.
/// Thus, several [Combo]s may match the same [KeyInput]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Combo {
    spec: KeySpec,
    modifiers: Vec<Modifier>,
}

impl From<KeySpec> for Combo {
    fn from(spec: KeySpec) -> Self {
        Combo {
            spec,
            modifiers: Vec::new(),
        }
    }
}

impl Combo {
    /// Check if a Combo matches a given key-input
    pub fn matches(&self, key_input: &KeyInput) -> bool {
        let key_matches = match &self.spec {
            KeySpec::Raw(x) => *x == key_input.code,
            KeySpec::Str(x) => *x == key_input.key,
        };
        key_matches && self.modifiers == key_input.modifiers
    }

    /// Turn a [KeyInput] into a corresponding [Combo] by looking at the raw key code
    pub fn from_keyinput_raw(input: KeyInput) -> Self {
        Self {
            modifiers: input.modifiers,
            spec: KeySpec::Raw(input.code),
        }
    }

    /// Turn a [KeyInput] into a corresponding [Combo] by looking at the key attribute value
    pub fn from_keyinput_str(input: KeyInput) -> Self {
        Self {
            modifiers: input.modifiers,
            spec: KeySpec::Str(input.key),
        }
    }
}

impl std::ops::Add<Modifier> for Combo {
    type Output = Combo;

    fn add(mut self, rhs: Modifier) -> Self::Output {
        self.modifiers.push(rhs);
        self
    }
}

impl From<Key> for Combo {
    fn from(key: Key) -> Self {
        Self {
            modifiers: Vec::new(),
            spec: KeySpec::Str(key),
        }
    }
}

impl From<RawKey> for Combo {
    fn from(key: RawKey) -> Self {
        Self {
            modifiers: Vec::new(),
            spec: KeySpec::Raw(key),
        }
    }
}

impl std::fmt::Display for Combo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.spec)
        } else {
            write!(f, "<{}-{}>", self.modifiers.iter().join("-"), self.spec)
        }
    }
}

impl FromStr for Combo {
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
                        spec: KeySpec::Raw(part.into()),
                    });
                }
            }
            Err(KeyInputParseError::EmptyInput)
        } else {
            Ok(Self {
                modifiers: Vec::new(),
                spec: KeySpec::Str(Key(s.to_string())),
            })
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum KeyInputParseError {
    #[error("Could not parse modifier {0}")]
    InvalidModifier(String),
    #[error("Input was empty")]
    EmptyInput,
}

#[cfg(test)]
mod test {
    use crate::{
        input_event::{Key, Modifier},
        key_combo::{Combo, KeySpec},
    };

    #[test]
    fn test_parse_combo() {
        let str_combo = |modifiers, key: &str| Combo {
            modifiers,
            spec: KeySpec::Str(Key(key.to_string())),
        };
        let raw_combo = |modifiers, key: &str| Combo {
            modifiers,
            spec: KeySpec::Raw(key.into()),
        };
        assert_eq!(str_combo(vec![], "Backspace"), "Backspace".parse().unwrap());
        assert_eq!(
            raw_combo(vec![], "Backspace"),
            "<Backspace>".parse().unwrap(),
        );
        assert_eq!(
            raw_combo(vec![Modifier::Ctrl], "Backspace"),
            "<C-Backspace>".parse().unwrap(),
        );
        assert_eq!(
            raw_combo(
                vec![Modifier::Ctrl, Modifier::Shift, Modifier::Alt],
                "Backspace"
            ),
            "<C-S-A-Backspace>".parse().unwrap(),
        );
        assert_eq!(raw_combo(vec![], "C"), "<C>".parse().unwrap());
    }
}
