//! Map [KeyInput]s to [user_buffer_op]-Operations

use bazed_rpc::keycode::{Key, KeyInput};

use crate::user_buffer_op::{BufferOp, DocumentOp, Motion, Operation};

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
        let op = match input.key {
            Key::Char(c) if input.shift_held() => {
                Buffer(BufferOp::Insert(c.to_ascii_lowercase().to_string()))
            },
            Key::Char(c) => Buffer(BufferOp::Insert(c.to_string())),
            Key::Backspace => Buffer(BufferOp::Backspace),
            // TODO we'll probably need to special-case this
            Key::Return => Buffer(BufferOp::Insert("\n".to_string())),
            // TODO we'll _definitely_ need to special case this for soft tabs
            Key::Tab => Buffer(BufferOp::Insert("\t".to_string())),

            Key::Left if input.shift_held() => Buffer(BufferOp::Selection(Motion::Left)),
            Key::Right if input.shift_held() => Buffer(BufferOp::Selection(Motion::Right)),
            Key::Up if input.shift_held() => Buffer(BufferOp::Selection(Motion::Up)),
            Key::Down if input.shift_held() => Buffer(BufferOp::Selection(Motion::Down)),

            Key::Left => Buffer(BufferOp::Move(Motion::Left)),
            Key::Right => Buffer(BufferOp::Move(Motion::Right)),
            Key::Up => Buffer(BufferOp::Move(Motion::Up)),
            Key::Down => Buffer(BufferOp::Move(Motion::Down)),
            _ => return None,
        };
        Some(op)
    }
}
