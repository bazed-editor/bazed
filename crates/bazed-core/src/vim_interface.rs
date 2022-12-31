use bazed_rpc::keycode::{Key, KeyInput};

use crate::{
    buffer::Buffer,
    user_buffer_op::{BufferOp, Motion, Trajectory},
    view::View,
    word_boundary::WordBoundaryType,
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, derive_more::Display)]
enum VimMode {
    #[default]
    Normal,
    Insert,
    Visual,
}

#[derive(Debug, Default)]
pub(crate) struct VimInterface {
    mode: VimMode,
}

impl VimInterface {
    pub(crate) fn on_input(&mut self, view: &View, buffer: &mut Buffer, input: KeyInput) {
        match self.mode {
            VimMode::Normal => self.on_input_normal_mode(view, buffer, &input),
            VimMode::Insert => self.on_input_insert_mode(view, buffer, &input),
            VimMode::Visual => self.on_input_visual_mode(view, buffer, &input),
        }
    }

    pub(crate) fn on_input_normal_mode(
        &mut self,
        view: &View,
        buffer: &mut Buffer,
        input: &KeyInput,
    ) {
        if let Some(motion) = key_to_motion_normal(input.ctrl_held(), &input.key) {
            buffer.apply_buffer_op(&view.vp, BufferOp::Move(motion));
            return;
        }
        match input.key {
            Key::Char('i') => self.switch_mode(VimMode::Insert),
            Key::Char('v') => self.switch_mode(VimMode::Visual),
            Key::Char('w') => buffer.apply_buffer_op(
                &view.vp,
                BufferOp::Move(Motion::NextWordBoundary(WordBoundaryType::Start)),
            ),
            Key::Char('b') => buffer.apply_buffer_op(
                &view.vp,
                BufferOp::Move(Motion::PrevWordBoundary(WordBoundaryType::Start)),
            ),
            Key::Char('e') => buffer.apply_buffer_op(
                &view.vp,
                BufferOp::Move(Motion::NextWordBoundary(WordBoundaryType::End)),
            ),
            Key::Char('x') => {
                buffer.apply_buffer_op(&view.vp, BufferOp::Delete(Trajectory::Forwards))
            },
            Key::Char('u') => buffer.apply_buffer_op(&view.vp, BufferOp::Undo),
            Key::Char('r') if input.ctrl_held() => buffer.apply_buffer_op(&view.vp, BufferOp::Redo),
            Key::Char('0') => buffer.apply_buffer_op(&view.vp, BufferOp::Move(Motion::StartOfLine)),
            Key::Char('$') => buffer.apply_buffer_op(&view.vp, BufferOp::Move(Motion::EndOfLine)),
            _ => {},
        }
    }

    pub(crate) fn on_input_insert_mode(
        &mut self,
        view: &View,
        buffer: &mut Buffer,
        input: &KeyInput,
    ) {
        if self.on_movement_key(view, buffer, &input) {
            return;
        }
        match input.key {
            Key::Char(c) => buffer.apply_buffer_op(&view.vp, BufferOp::Insert(c.to_string())),
            Key::Backspace => {
                buffer.apply_buffer_op(&view.vp, BufferOp::Delete(Trajectory::Backwards))
            },
            Key::Delete => buffer.apply_buffer_op(&view.vp, BufferOp::Delete(Trajectory::Forwards)),
            Key::Return => buffer.apply_buffer_op(&view.vp, BufferOp::Insert("\n".to_string())),
            Key::Tab => buffer.apply_buffer_op(&view.vp, BufferOp::Insert("\t".to_string())),
            Key::Escape => self.switch_mode(VimMode::Normal),
            _ => {},
        }
    }

    pub(crate) fn on_input_visual_mode(
        &mut self,
        view: &View,
        buffer: &mut Buffer,
        input: &KeyInput,
    ) {
        if let Some(motion) = key_to_motion_normal(input.ctrl_held(), &input.key) {
            buffer.apply_buffer_op(&view.vp, BufferOp::Selection(motion));
            return;
        }
        match input.key {
            Key::Escape => {
                self.mode = VimMode::Normal;
                buffer.collapse_selections();
            },
            Key::Char('d') | Key::Char('x') => {
                buffer.apply_buffer_op(&view.vp, BufferOp::DeleteSelected);
            },
            _ => {
                if let Some(motion) = key_to_motion(input.ctrl_held(), &input.key) {
                    buffer.apply_buffer_op(&view.vp, BufferOp::Selection(motion));
                }
            },
        }
    }

    /// Handle regular movement keys (usable for normal and insert mode)
    fn on_movement_key(&mut self, view: &View, buffer: &mut Buffer, input: &KeyInput) -> bool {
        let Some(motion) = key_to_motion(input.ctrl_held(), &input.key)  else {
            return false;
        };
        let op = if input.alt_held() {
            BufferOp::NewCaret(motion)
        } else {
            BufferOp::Move(motion)
        };
        buffer.apply_buffer_op(&view.vp, op);
        true
    }

    fn switch_mode(&mut self, mode: VimMode) {
        self.mode = mode;
    }
}

/// Map a movement key into the corresponding [Motion], assuming normal mode
fn key_to_motion_normal(ctrl_held: bool, key: &Key) -> Option<Motion> {
    if let Some(motion) = key_to_motion(ctrl_held, key) {
        return Some(motion);
    }
    return Some(match key {
        Key::Char('w') => Motion::NextWordBoundary(WordBoundaryType::Start),
        Key::Char('b') => Motion::PrevWordBoundary(WordBoundaryType::Start),
        Key::Char('e') => Motion::NextWordBoundary(WordBoundaryType::End),

        Key::Char('h') => Motion::Left,
        Key::Char('l') => Motion::Right,
        Key::Char('k') => Motion::Up,
        Key::Char('j') => Motion::Down,
        Key::Char('0') => Motion::StartOfLine,
        Key::Char('$') => Motion::EndOfLine,
        _ => return None,
    });
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
