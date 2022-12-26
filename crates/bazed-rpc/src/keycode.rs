use serde::{Deserialize, Serialize};

/// A combination of held [Modifier]s and a [Key].
// TODO figure out normalization: Do we get `Shift+a` or do we get `Key::Char('A')`?
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
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

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Win,
}

/// Any relevant letter, symbol of nav-key on a standard qwerty keyboard.
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Key {
    Char(char),
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Backspace,
    Return,
    Tab,
    Home,
    End,
    Insert,
    Delete,
    PageUp,
    PageDown,
    Escape,
    Left,
    Right,
    Up,
    Down,
}

impl Key {
    pub fn try_as_char(&self) -> Option<char> {
        match self {
            Self::Char(c) => Some(*c),
            _ => None,
        }
    }
}
