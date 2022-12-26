//! Map [KeyInput]s to [user_buffer_op]-Operations

use bazed_rpc::keycode::{Key, KeyInput};

use crate::user_buffer_op::{BufferOp, DocumentOp, Motion, Operation, Trajectory};

pub(crate) fn interpret_key_input(input: &KeyInput) -> Option<Operation> {
    if input.ctrl_held() {
        match input.key {
            Key::Char('z') => Some(Operation::Buffer(BufferOp::Undo)),
            Key::Char('y') => Some(Operation::Buffer(BufferOp::Redo)),
            Key::Char('s') => Some(Operation::Document(DocumentOp::Save)),
            _ => None,
        }
    } else {
        use Operation::*;

        // for now we just ignore the fact that other modifiers like Alt and Win exist.
        Some(match input.key {
            Key::Char(c) if input.shift_held() => {
                Buffer(BufferOp::Insert(c.to_ascii_lowercase().to_string()))
            },
            Key::Char(c) => Buffer(BufferOp::Insert(c.to_string())),
            Key::Backspace => Buffer(BufferOp::Delete(Trajectory::Backwards)),
            Key::Delete => Buffer(BufferOp::Delete(Trajectory::Forwards)),
            Key::Return => Buffer(BufferOp::Insert("\n".to_string())),
            Key::Tab => Buffer(BufferOp::Insert("\t".to_string())),

            _ => match key_to_motion(&input.key) {
                Some(motion) if input.shift_held() => Buffer(BufferOp::Selection(motion)),
                Some(motion) if input.modifiers.is_empty() => Buffer(BufferOp::Move(motion)),
                _ => return None,
            },
        })
    }
}

/// Map a movement key into the corresponding [Motion].
/// This most likely won't scale to our future architecture, but it works for now
fn key_to_motion(key: &Key) -> Option<Motion> {
    match key {
        Key::Left => Some(Motion::Left),
        Key::Right => Some(Motion::Right),
        Key::Up => Some(Motion::Up),
        Key::Down => Some(Motion::Down),
        Key::Home => Some(Motion::StartOfLine),
        Key::End => Some(Motion::EndOfLine),
        _ => None,
    }
}
