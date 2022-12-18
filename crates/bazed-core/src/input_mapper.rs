//! Map [KeyInput]s to [user_buffer_op]-Operations

use bazed_rpc::keycode::{Key, KeyInput};

use crate::user_buffer_op::{BufferOp, EditOp, MovementOp};

pub(crate) fn interpret_key_input(input: &KeyInput) -> Option<BufferOp> {
    if input.ctrl_held() {
        match input.key {
            Key::Char('z') => Some(BufferOp::Edit(EditOp::Undo)),
            _ => None,
        }
    } else {
        // for now we just ignore the fact that other modifiers like Alt and Win exist.
        match input.key {
            Key::Char(c) if input.shift_held() => Some(BufferOp::Edit(EditOp::Insert(
                c.to_ascii_lowercase().to_string(),
            ))),
            Key::Char(c) => Some(BufferOp::Edit(EditOp::Insert(c.to_string()))),
            Key::Backspace => Some(BufferOp::Edit(EditOp::Backspace)),
            // TODO we'll probably need to special-case this
            Key::Return => Some(BufferOp::Edit(EditOp::Insert("\n".to_string()))),
            // TODO we'll _definitely_ need to special case this for soft tabs
            Key::Tab => Some(BufferOp::Edit(EditOp::Insert("\t".to_string()))),
            Key::Left => Some(BufferOp::Movement(MovementOp::Left)),
            Key::Right => Some(BufferOp::Movement(MovementOp::Right)),
            Key::Up => Some(BufferOp::Movement(MovementOp::Up)),
            Key::Down => Some(BufferOp::Movement(MovementOp::Down)),
            _ => None,
        }
    }
}
