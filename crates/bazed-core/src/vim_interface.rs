use std::{collections::HashMap, sync::Arc};

use bazed_input_mapper::{
    input_event::{Key, KeyInput, Modifiers},
    key_combo::{Combo, KeySpec},
    keymap::{Keymap, KeymapNode},
    InputMapper, KeymapId,
};

use crate::{
    buffer::Buffer,
    user_buffer_op::{BufferOp, Motion, Trajectory},
    view::View,
    word_boundary::WordBoundaryType,
};

type MappedFn =
    Arc<Box<dyn Fn(&View, &mut Buffer, &mut VimInterface, KeyInput) + Send + Sync + 'static>>;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, derive_more::Display)]
pub enum VimMode {
    #[default]
    Normal,
    Insert,
    Visual,
    Replace,
}

impl VimMode {
    fn keymap_id(&self) -> KeymapId {
        let s = match self {
            VimMode::Normal => "vim/normal",
            VimMode::Insert => "vim/insert",
            VimMode::Visual => "vim/visual",
            VimMode::Replace => "vim/replace",
        };
        KeymapId(s.to_string())
    }

    fn corresponding_keymap(&self) -> Keymap<MappedFn> {
        match self {
            VimMode::Normal => normal_mode_keymap(),
            VimMode::Insert => insert_mode_keymap(),
            VimMode::Visual => visual_mode_keymap(),
            VimMode::Replace => replace_mode_keymap(),
        }
    }
}

pub(crate) struct VimInterface {
    pub(crate) input_mapper: InputMapper<MappedFn>,
    pub(crate) mode: VimMode,
}

impl VimInterface {
    pub(crate) fn new() -> Self {
        let mut input_mapper = InputMapper::from_base_keymap(
            KeymapId("vim-mode/empty".to_string()),
            Keymap::new_from_map(HashMap::from_iter([(
                key("Escape"),
                leaf("Normal mode", |_, _, vim, _| {
                    tracing::warn!("Reached base keymap of vim, this shouldn't really happen");
                    vim.switch_mode(VimMode::Normal)
                }),
            )])),
        );
        input_mapper.register_keymap(
            VimMode::Normal.keymap_id(),
            VimMode::Normal.corresponding_keymap(),
        );
        input_mapper.register_keymap(
            VimMode::Insert.keymap_id(),
            VimMode::Insert.corresponding_keymap(),
        );
        input_mapper.register_keymap(
            VimMode::Visual.keymap_id(),
            VimMode::Visual.corresponding_keymap(),
        );
        input_mapper.register_keymap(
            VimMode::Replace.keymap_id(),
            VimMode::Replace.corresponding_keymap(),
        );
        _ = input_mapper.push_keymap(VimMode::Normal.keymap_id());
        Self {
            input_mapper,
            mode: VimMode::Normal,
        }
    }

    #[tracing::instrument(skip_all, fields(mode = %self.mode))]
    pub(crate) fn on_input(&mut self, view: &View, buffer: &mut Buffer, input: KeyInput) {
        match self.input_mapper.on_input(input.clone()) {
            Some(KeymapNode::Leaf(_, f)) => f.clone()(view, buffer, self, input),
            Some(KeymapNode::Submap(x, _)) => tracing::info!("In submap {x}"),
            None => tracing::info!("No mapping for {input}"),
        }
    }

    fn switch_mode(&mut self, mode: VimMode) {
        self.input_mapper.deactivate_keymap(self.mode.keymap_id());
        if let Err(err) = self.input_mapper.push_keymap(mode.keymap_id()) {
            tracing::error!("Error switching vim mode: {err}");
        }
        self.mode = mode;
    }
}

pub(crate) fn replace_mode_keymap() -> Keymap<MappedFn> {
    let on_printable: MappedFn = Arc::new(Box::new(|_, b, _, k| {
        b.replace_at_carets(&k.to_string());
    }));
    Keymap::new(
        HashMap::from_iter([(
            key("Escape"),
            leaf("normal mode", |_, _, vim, _| {
                vim.switch_mode(VimMode::Normal)
            }),
        )]),
        Some(KeymapNode::Leaf("insert".to_string(), on_printable)),
    )
}

pub(crate) fn normal_mode_keymap() -> Keymap<MappedFn> {
    normal_mode_movement_key_keymap().merge(Keymap::new_from_map(HashMap::from_iter([
        (
            key("i"),
            leaf("insert mode", |_, _, vim, _| {
                vim.switch_mode(VimMode::Insert)
            }),
        ),
        (
            key("n").with_mods(Modifiers::ALT),
            KeymapNode::Submap("new caret".to_string(), Box::new(add_caret_keymap())),
        ),
        (
            key("v"),
            leaf("visual mode", |_, _, vim, _| {
                vim.switch_mode(VimMode::Visual)
            }),
        ),
        (
            key("r").with_mods(Modifiers::SHIFT),
            leaf("replace mode", |_, _, vim, _| {
                vim.switch_mode(VimMode::Replace)
            }),
        ),
        (
            key("x"),
            leaf("", |v, b, _, _| {
                b.apply_buffer_op(&v.vp, BufferOp::Delete(Trajectory::Forwards))
            }),
        ),
        (
            key("u"),
            leaf("", |v, b, _, _| b.apply_buffer_op(&v.vp, BufferOp::Undo)),
        ),
        (
            key("r").with_mods(Modifiers::CTRL),
            leaf("", |v, b, _, _| b.apply_buffer_op(&v.vp, BufferOp::Redo)),
        ),
        (
            key("0"),
            leaf("", |v, b, _, _| {
                b.apply_buffer_op(&v.vp, BufferOp::Move(Motion::StartOfLine))
            }),
        ),
        (
            translated_key("$"),
            leaf("", |v, b, _, _| {
                b.apply_buffer_op(&v.vp, BufferOp::Move(Motion::EndOfLine))
            }),
        ),
    ])))
}

fn insert_mode_keymap() -> Keymap<MappedFn> {
    let on_printable: MappedFn = mapping(|v, b: &mut Buffer, _, k: KeyInput| {
        b.apply_buffer_op(&v.vp, BufferOp::Insert(k.key.to_string()));
    });
    movement_key_keymap().merge(Keymap::new(
        HashMap::from_iter([
            (
                key("Backspace"),
                leaf("backspace", |v, b, _, _| {
                    b.apply_buffer_op(&v.vp, BufferOp::Delete(Trajectory::Backwards))
                }),
            ),
            (
                key("Delete"),
                leaf("backspace", |v, b, _, _| {
                    b.apply_buffer_op(&v.vp, BufferOp::Delete(Trajectory::Forwards))
                }),
            ),
            (
                key("Enter"),
                leaf("backspace", |v, b, _, _| {
                    b.apply_buffer_op(&v.vp, BufferOp::Insert("\n".to_string()))
                }),
            ),
            (
                key("Tab"),
                leaf("backspace", |v, b, _, _| {
                    b.apply_buffer_op(&v.vp, BufferOp::Insert("\t".to_string()))
                }),
            ),
            (
                key("Escape"),
                leaf("normal mode", |_, _, vim, _| {
                    vim.switch_mode(VimMode::Normal)
                }),
            ),
        ]),
        Some(KeymapNode::Leaf("type".to_string(), on_printable)),
    ))
}

fn visual_mode_keymap() -> Keymap<MappedFn> {
    let visual_mode_movement = normal_mode_movement_key_motion_keymap().map(&|motion| {
        mapping(move |v, b, _, _| b.apply_buffer_op(&v.vp, BufferOp::Selection(motion)))
    });
    let keymap = Keymap::new_from_map(HashMap::from_iter([
        (
            key("Escape"),
            leaf("normal mode", |_, b, vim, _| {
                vim.switch_mode(VimMode::Normal);
                b.collapse_selections();
            }),
        ),
        (
            key("d"),
            leaf("delete", |v, b, _, _| {
                b.apply_buffer_op(&v.vp, BufferOp::DeleteSelected);
            }),
        ),
        (
            key("x"),
            leaf("delete", |v, b, _, _| {
                b.apply_buffer_op(&v.vp, BufferOp::DeleteSelected);
            }),
        ),
    ]));
    visual_mode_movement.merge(keymap)
}

fn add_caret_keymap() -> Keymap<MappedFn> {
    normal_mode_movement_key_motion_keymap().map(&|motion: Motion| {
        mapping(move |v, b, _, _| b.apply_buffer_op(&v.vp, BufferOp::NewCaret(motion)))
    })
}

fn normal_mode_movement_key_keymap() -> Keymap<MappedFn> {
    normal_mode_movement_key_motion_keymap().map(&|motion: Motion| {
        mapping(move |v, b, _, _| b.apply_buffer_op(&v.vp, BufferOp::Move(motion)))
    })
}

fn movement_key_keymap() -> Keymap<MappedFn> {
    movement_key_motion_keymap()
        .map(&|motion| mapping(move |v, b, _, _| b.apply_buffer_op(&v.vp, BufferOp::Move(motion))))
}

fn normal_mode_movement_key_motion_keymap() -> Keymap<Motion<'static>> {
    let normal_mode_movement_binds = Keymap::new_from_map(HashMap::from_iter([
        (
            key("w"),
            KeymapNode::Leaf(
                "to next word".to_string(),
                Motion::NextWordBoundary(WordBoundaryType::Start),
            ),
        ),
        (
            key("b"),
            KeymapNode::Leaf(
                "to previous word".to_string(),
                Motion::PrevWordBoundary(WordBoundaryType::Start),
            ),
        ),
        (key("h"), KeymapNode::Leaf("left".to_string(), Motion::Left)),
        (
            key("l"),
            KeymapNode::Leaf("right".to_string(), Motion::Right),
        ),
        (key("k"), KeymapNode::Leaf("up".to_string(), Motion::Up)),
        (key("j"), KeymapNode::Leaf("down".to_string(), Motion::Down)),
        (
            key("0"),
            KeymapNode::Leaf("to start of line".to_string(), Motion::StartOfLine),
        ),
        (
            translated_key("$"),
            KeymapNode::Leaf("to end of line".to_string(), Motion::EndOfLine),
        ),
    ]));
    normal_mode_movement_binds.merge(movement_key_motion_keymap())
}

fn movement_key_motion_keymap() -> Keymap<Motion<'static>> {
    Keymap::new_from_map(HashMap::from_iter([
        (
            key("ArrowRight").with_mods(Modifiers::CTRL),
            KeymapNode::Leaf(
                "to next word".to_string(),
                Motion::NextWordBoundary(WordBoundaryType::Start),
            ),
        ),
        (
            key("ArrowLeft").with_mods(Modifiers::CTRL),
            KeymapNode::Leaf(
                "to previous word".to_string(),
                Motion::PrevWordBoundary(WordBoundaryType::Start),
            ),
        ),
        (
            key("ArrowLeft"),
            KeymapNode::Leaf("left".to_string(), Motion::Left),
        ),
        (
            key("ArrowRight"),
            KeymapNode::Leaf("right".to_string(), Motion::Right),
        ),
        (
            key("ArrowUp"),
            KeymapNode::Leaf("up".to_string(), Motion::Up),
        ),
        (
            key("ArrowDown"),
            KeymapNode::Leaf("down".to_string(), Motion::Down),
        ),
        (
            key("Home"),
            KeymapNode::Leaf("to start of line".to_string(), Motion::StartOfLine),
        ),
        (
            key("End"),
            KeymapNode::Leaf("to end of line".to_string(), Motion::EndOfLine),
        ),
    ]))
}

fn key(k: &str) -> Combo {
    Combo::from(KeySpec::Raw(k.into()))
}
fn translated_key(k: &str) -> Combo {
    Combo::from(KeySpec::Str(Key(k.to_string())))
}

fn leaf<F: Fn(&View, &mut Buffer, &mut VimInterface, KeyInput) + Send + Sync + 'static>(
    desc: &str,
    f: F,
) -> KeymapNode<MappedFn> {
    KeymapNode::Leaf(desc.to_string(), mapping(f))
}

fn mapping<F: Fn(&View, &mut Buffer, &mut VimInterface, KeyInput) + Send + Sync + 'static>(
    f: F,
) -> MappedFn {
    Arc::new(Box::new(f))
}
