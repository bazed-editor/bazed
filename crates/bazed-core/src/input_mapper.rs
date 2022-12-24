//! Map [KeyInput]s to [user_buffer_op]-Operations

use bazed_rpc::keycode::{Key, KeyInput};

use crate::user_buffer_op::{DocumentOp, EditOp, MovementOp, Operation};

pub(crate) fn interpret_key_input(input: &KeyInput) -> Option<Operation> {
    if input.ctrl_held() {
        match input.key {
            Key::Char('z') => Some(Operation::Edit(EditOp::Undo)),
            Key::Char('y') => Some(Operation::Edit(EditOp::Redo)),
            Key::Char('s') => Some(Operation::Document(DocumentOp::Save)),
            _ => None,
        }
    } else {
        // for now we just ignore the fact that other modifiers like Alt and Win exist.
        match input.key {
            Key::Char(c) if input.shift_held() => Some(Operation::Edit(EditOp::Insert(
                c.to_ascii_lowercase().to_string(),
            ))),
            Key::Char(c) => Some(Operation::Edit(EditOp::Insert(c.to_string()))),
            Key::Backspace => Some(Operation::Edit(EditOp::Backspace)),
            // TODO we'll probably need to special-case this
            Key::Return => Some(Operation::Edit(EditOp::Insert("\n".to_string()))),
            // TODO we'll _definitely_ need to special case this for soft tabs
            Key::Tab => Some(Operation::Edit(EditOp::Insert("\t".to_string()))),
            Key::Left => Some(Operation::Movement(MovementOp::Left)),
            Key::Right => Some(Operation::Movement(MovementOp::Right)),
            Key::Up => Some(Operation::Movement(MovementOp::Up)),
            Key::Down => Some(Operation::Movement(MovementOp::Down)),
            _ => None,
        }
    }
}
