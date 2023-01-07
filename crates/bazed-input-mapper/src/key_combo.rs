//! Key combo definitions as they are used in specifying keymaps.

use std::str::FromStr;

use crate::input_event::{Key, KeyInput, Modifiers, RawKey};

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
    modifiers: Modifiers,
}

impl From<KeySpec> for Combo {
    fn from(spec: KeySpec) -> Self {
        Combo {
            spec,
            modifiers: Modifiers::empty(),
        }
    }
}

impl Combo {
    /// Check if a Combo matches a given key-input
    pub fn matches(&self, key_input: &KeyInput) -> bool {
        match &self.spec {
            KeySpec::Raw(x) => *x == key_input.code && self.modifiers == key_input.modifiers,
            KeySpec::Str(x) => *x == key_input.key && key_input.modifiers.contains(self.modifiers),
        }
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

    pub fn with_mods(mut self, mods: Modifiers) -> Self {
        self.modifiers = mods;
        self
    }
}

impl From<Key> for Combo {
    fn from(key: Key) -> Self {
        Self {
            modifiers: Modifiers::empty(),
            spec: KeySpec::Str(key),
        }
    }
}

impl From<RawKey> for Combo {
    fn from(key: RawKey) -> Self {
        Self {
            modifiers: Modifiers::empty(),
            spec: KeySpec::Raw(key),
        }
    }
}

impl std::fmt::Display for Combo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.modifiers.is_empty() {
            write!(f, "{}", self.spec)
        } else {
            write!(f, "<{}-{}>", self.modifiers, self.spec)
        }
    }
}

impl FromStr for Combo {
    type Err = KeyInputParseError;

    // TODO related to normalization: Do we wanna parse `A` as `<S-A>`, `A` or `<S-a>`?
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(s) = s.strip_prefix('<').and_then(|s| s.strip_suffix('>')) {
            let mut parts = s.split('-').peekable();
            let mut modifiers = Modifiers::empty();
            while let Some(part) = parts.next() {
                if parts.peek().is_some() {
                    modifiers |= part
                        .chars()
                        .next()
                        .and_then(Modifiers::from_char)
                        .ok_or_else(|| KeyInputParseError::InvalidModifier(part.to_string()))?;
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
                modifiers: Modifiers::empty(),
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
        input_event::{Key, Modifiers},
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
        assert_eq!(
            str_combo(Modifiers::empty(), "Backspace"),
            "Backspace".parse().unwrap()
        );
        assert_eq!(
            raw_combo(Modifiers::empty(), "Backspace"),
            "<Backspace>".parse().unwrap(),
        );
        assert_eq!(
            raw_combo(Modifiers::CTRL, "Backspace"),
            "<C-Backspace>".parse().unwrap(),
        );
        assert_eq!(
            raw_combo(
                Modifiers::CTRL | Modifiers::SHIFT | Modifiers::ALT,
                "Backspace"
            ),
            "<C-S-A-Backspace>".parse().unwrap(),
        );
        assert_eq!(raw_combo(Modifiers::empty(), "C"), "<C>".parse().unwrap());
    }
}
