use serde::{Deserialize, Serialize};

/// A combination of held [Modifier]s and a [Key].
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyInput {
    pub modifiers: Vec<Modifier>,
    pub key: Key,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Modifier {
    Ctrl,
    Alt,
    Shift,
    Win,
}

/// Any relevant letter, symbol of nav-key on a standard qwerty keyboard.
#[derive(Debug, Serialize, Deserialize)]
pub enum Key {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Tilde,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
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
    Minus,
    Plus,
    Backspace,
    Tab,
    LBracket,
    RBracket,
    Backslash,
    Semicolon,
    Quote,
    Return,
    Comma,
    Period,
    Slash,
    Space,
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
    pub fn try_into_char(self) -> Option<char> {
        Some(match self {
            Self::A => 'a',
            Self::B => 'b',
            Self::C => 'c',
            Self::D => 'd',
            Self::E => 'e',
            Self::F => 'f',
            Self::G => 'g',
            Self::H => 'h',
            Self::I => 'i',
            Self::J => 'j',
            Self::K => 'k',
            Self::L => 'l',
            Self::M => 'm',
            Self::N => 'n',
            Self::O => 'o',
            Self::P => 'p',
            Self::Q => 'q',
            Self::R => 'r',
            Self::S => 's',
            Self::T => 't',
            Self::U => 'u',
            Self::V => 'v',
            Self::W => 'w',
            Self::X => 'x',
            Self::Y => 'y',
            Self::Z => 'z',
            _ => return None,
        })
    }
}