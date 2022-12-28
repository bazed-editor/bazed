//! Map [KeyInput]s to [user_buffer_op]-Operations

use bazed_rpc::keycode::{Key, KeyInput};

use crate::{
    user_buffer_op::{BufferOp, DocumentOp, Motion, Operation, Trajectory},
    word_boundary::WordBoundaryType,
};

pub(crate) fn interpret_key_input(input: &KeyInput) -> Option<Operation> {
    Some(match input.key {
        Key::Char(c) if input.ctrl_held() => match c {
            'z' => Operation::Buffer(BufferOp::Undo),
            'y' => Operation::Buffer(BufferOp::Redo),
            's' => Operation::Document(DocumentOp::Save),
            _ => return None,
        },
        Key::Char(c) if input.shift_held() => {
            Operation::Buffer(BufferOp::Insert(c.to_ascii_lowercase().to_string()))
        },
        Key::Char(c) => Operation::Buffer(BufferOp::Insert(c.to_string())),
        Key::Backspace => Operation::Buffer(BufferOp::Delete(Trajectory::Backwards)),
        Key::Delete => Operation::Buffer(BufferOp::Delete(Trajectory::Forwards)),
        Key::Return => Operation::Buffer(BufferOp::Insert("\n".to_string())),
        Key::Tab => Operation::Buffer(BufferOp::Insert("\t".to_string())),

        _ => match key_to_motion(input.ctrl_held(), &input.key) {
            Some(motion) if input.shift_held() => Operation::Buffer(BufferOp::Selection(motion)),
            Some(motion) if input.alt_held() => Operation::Buffer(BufferOp::NewCaret(motion)),
            Some(motion) => Operation::Buffer(BufferOp::Move(motion)),
            _ => return None,
        },
    })
}

/// Map a movement key into the corresponding [Motion].
/// This most likely won't scale to our future architecture, but it works for now
fn key_to_motion(ctrl_held: bool, key: &Key) -> Option<Motion> {
    match key {
        Key::Right if ctrl_held => Some(Motion::NextWordBoundary(WordBoundaryType::Start)),
        Key::Left if ctrl_held => Some(Motion::PrevWordBoundary(WordBoundaryType::Start)),

        Key::Left => Some(Motion::Left),
        Key::Right => Some(Motion::Right),
        Key::Up => Some(Motion::Up),
        Key::Down => Some(Motion::Down),
        Key::Home => Some(Motion::StartOfLine),
        Key::End => Some(Motion::EndOfLine),
        _ => None,
    }
}
