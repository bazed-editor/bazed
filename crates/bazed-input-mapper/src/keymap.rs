use std::collections::HashMap;

use crate::input_event::{KeyInput, Modifier};

pub struct Keymap<V> {
    map: HashMap<KeyInput, KeymapNode<V>>,
    /// If no other map was matched, but the pressed key is a printable character
    /// (corresponding to <https://www.w3.org/TR/uievents-key/#key-string>)
    /// this node will be matched
    on_any_printable: Option<KeymapNode<V>>,
}

pub enum KeymapNode<V> {
    Submap(String, Box<Keymap<V>>),
    Leaf(String, V),
}

impl<V> KeymapNode<V> {
    pub fn description(&self) -> &str {
        match self {
            KeymapNode::Submap(x, _) | KeymapNode::Leaf(x, _) => x,
        }
    }
}

impl<V> Keymap<V> {
    pub fn node_at_input(&self, input: &KeyInput) -> Option<&KeymapNode<V>> {
        self.map.get(input).or_else(|| {
            self.on_any_printable
                .as_ref()
                .filter(|_| input_is_printable(input))
        })
    }

    pub fn node_at_path(&self, inputs: &[KeyInput]) -> Option<&KeymapNode<V>> {
        let next = inputs.first()?;
        match self.node_at_input(next)? {
            KeymapNode::Submap(_, submap) => submap.node_at_path(&inputs[1..]),
            leaf => Some(leaf),
        }
    }
}

fn input_is_printable(input: &KeyInput) -> bool {
    (input.modifiers.is_empty() || input.modifiers == [Modifier::Shift])
        && input.key.is_key_string()
}
