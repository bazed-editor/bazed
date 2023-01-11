#![forbid(unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::private_intra_doc_links)]
//! The input mapper is the primary component that maps input-events to functions and other, semantic events.
//! It manages loading the configuration and the logic for interpreting key inputs (i.e. detecting input chains).
//!
//! It does not however include any key interpretation of its own. Concrete keymaps will rely on this component
//! for a lot of the input logic, but will manage their state themselves.

use std::collections::HashMap;

use input_event::KeyInput;
use keymap::{Keymap, KeymapNode};
use nonempty::NonEmpty;

pub mod input_event;
pub mod key_combo;
pub mod keymap;

/// Id of a keymap.
///
/// Represented by a string for now, but this will probably be changed to use some namespacing mechanism
/// and _then_ a proper name in the future
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct KeymapId(pub String);

/// The input mapper.
///
/// Keymaps have to be explicitly registered by ID first, and can then be activated later.
/// When a key input is received, the stack of active keymaps is traversed
/// until a match is found
#[derive(Debug)]
pub struct InputMapper<V> {
    /// Map of all registered keymaps.
    keymaps: HashMap<KeymapId, Keymap<V>>,
    /// The stack of currently active keymaps. Newly activated maps get pushed to the top.
    /// Key inputs get checked against the stack top to bottom, until a match is found.
    ///
    /// Invariant: KeymapIds always map to entries in `keymaps`
    stack: NonEmpty<KeymapId>,

    /// Input currently buffered. When the last pressed key mapped to some submap,
    /// that key will be buffered, such that further lookups can be done when the next input is received.
    buffered_inputs: Vec<KeyInput>,
}

impl<V> InputMapper<V> {
    pub fn from_base_keymap(keymap_id: KeymapId, keymap: Keymap<V>) -> Self {
        Self {
            keymaps: HashMap::from_iter([(keymap_id.clone(), keymap)]),
            stack: nonempty::nonempty![keymap_id],
            buffered_inputs: Vec::new(),
        }
    }
    pub fn register_keymap(&mut self, keymap_id: KeymapId, keymap: Keymap<V>) {
        self.keymaps.insert(keymap_id, keymap);
    }

    /// Activate a keymap. fails when no keymap with that id is registered
    pub fn push_keymap(&mut self, keymap_id: KeymapId) -> Result<(), Error> {
        if !self.keymaps.contains_key(&keymap_id) {
            return Err(Error::KeymapNotRegistered(keymap_id));
        }
        // TODO should we allow activating already active keymaps at all?
        // should already active keymaps just be pulled to the top of the stack, rather than duplicated?
        if self.stack.last() == &keymap_id {
            return Err(Error::KeymapAlreadyAtTop);
        }
        self.stack.push(keymap_id);
        Ok(())
    }

    /// Deactivate the top-most occurrence of the given keymap.
    /// **NOTE** that this will never deactivate the base map.
    pub fn deactivate_keymap(&mut self, keymap_id: KeymapId) {
        let last_entry = self
            .stack
            .tail
            .iter()
            .enumerate()
            .rev()
            .find(|(_, x)| x == &&keymap_id);
        if let Some((idx, _)) = last_entry {
            self.stack.tail.remove(idx);
        }
    }

    /// Handle a single key input.
    ///
    /// Buffers inputs when the input leads us to a submap.
    /// When we hit a leaf or no match at all, the buffered inputs are cleared.
    pub fn on_input(&mut self, input: KeyInput) -> Option<&KeymapNode<V>> {
        self.buffered_inputs.push(input);
        let active_keymap = self.keymaps.get(self.stack.last()).unwrap();
        let node = active_keymap.node_at_path(&self.buffered_inputs);
        match node {
            Some(x @ KeymapNode::Leaf(_, _)) => {
                self.buffered_inputs.clear();
                Some(x)
            },
            Some(x @ KeymapNode::Submap(_, _)) => Some(x),
            None => {
                self.buffered_inputs.clear();
                None
            },
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No keymap with id {0:?} registered")]
    KeymapNotRegistered(KeymapId),
    #[error("The keymap was already at the top of the stack")]
    KeymapAlreadyAtTop,
}
