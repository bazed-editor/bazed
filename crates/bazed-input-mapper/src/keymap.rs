//! Representation of a (possibly nested) keymap.

use std::collections::HashMap;

use crate::{
    input_event::{KeyInput, Modifier},
    key_combo::Combo,
};

/// A keymap specifies mappings from [Combo]s to some value (typically a callback or event).
/// Keymaps may be nested, meaning that a key is either mapped to a Submap, or to some concrete value.
///
/// Every keymap can optionally have a fallback case for printable characters.
#[derive(Debug)]
pub struct Keymap<V> {
    pub map: HashMap<Combo, KeymapNode<V>>,
    /// If no other map was matched, but the pressed key is a printable character
    /// (corresponding to <https://www.w3.org/TR/uievents-key/#key-string>)
    /// this node will be matched
    pub on_any_printable: Option<KeymapNode<V>>,
}

/// Either a submap, or a single value, used to represent the nested structure of a [Keymap].
/// Also includes a short description of the node, for use in debugging and user interfaces.
#[derive(Debug)]
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

    /// recursively map a function over the leaves of this node
    pub fn map<O>(self, f: &dyn Fn(V) -> O) -> KeymapNode<O> {
        match self {
            KeymapNode::Submap(d, submap) => KeymapNode::Submap(d, Box::new(submap.map(f))),
            KeymapNode::Leaf(d, v) => KeymapNode::Leaf(d, f(v)),
        }
    }

    /// Merge two keymap-nodes together recursively.
    /// If one of them is a leaf, `other` takes precedence.
    pub fn merge(self, other: KeymapNode<V>) -> Self {
        match (self, other) {
            (_, x @ Self::Leaf(_, _)) => x,
            (Self::Leaf(_, _), x) => x,
            (Self::Submap(_, m1), Self::Submap(d2, m2)) => {
                Self::Submap(d2, Box::new(m1.merge(*m2)))
            },
        }
    }
}

impl<V> Keymap<V> {
    pub fn new(
        map: HashMap<Combo, KeymapNode<V>>,
        on_any_printable: Option<KeymapNode<V>>,
    ) -> Self {
        Self {
            map,
            on_any_printable,
        }
    }

    pub fn new_from_map(map: HashMap<Combo, KeymapNode<V>>) -> Self {
        Self::new(map, None)
    }

    /// Merge two keymaps together recursively.
    /// If there are colliding mappings, `other` takes precedence
    pub fn merge(mut self, other: Keymap<V>) -> Self {
        let Self {
            map,
            on_any_printable,
        } = other;
        for (k, v) in map.into_iter() {
            match self.map.remove(&k) {
                Some(node) => {
                    self.map.insert(k, node.merge(v));
                },
                None => {
                    self.map.insert(k, v);
                },
            }
        }
        self.on_any_printable = on_any_printable.or(self.on_any_printable);
        self
    }

    pub fn descriptions(&self) -> impl Iterator<Item = (&Combo, &str)> {
        self.map.iter().map(|(k, v)| (k, v.description()))
    }

    /// recursively map a function over the leaves of this node
    pub fn map<O>(self, f: &dyn Fn(V) -> O) -> Keymap<O> {
        let map = self.map.into_iter().map(|(k, v)| (k, v.map(&f))).collect();
        let on_any_printable = self.on_any_printable.map(|v| v.map(f));
        Keymap {
            map,
            on_any_printable,
        }
    }

    /// Get the [KeymapNode] corresponding to the given input, if there is one
    pub fn node_at_input(&self, input: &KeyInput) -> Option<&KeymapNode<V>> {
        self.map
            .get(&Combo::from_keyinput_raw(input.clone()))
            .or_else(|| self.map.get(&Combo::from_keyinput_str(input.clone())))
            .or_else(|| {
                self.on_any_printable
                    .as_ref()
                    .filter(|_| input_is_printable(input))
            })
    }

    /// Get the [KeymapNode] corresponding to the given chain of inputs, if there is one
    pub fn node_at_path(&self, inputs: &[KeyInput]) -> Option<&KeymapNode<V>> {
        let next = inputs.first()?;
        match self.node_at_input(next)? {
            submap @ KeymapNode::Submap(_, _) if inputs.len() == 1 => Some(submap),
            KeymapNode::Submap(_, submap) => submap.node_at_path(&inputs[1..]),
            leaf => Some(leaf),
        }
    }
}

fn input_is_printable(input: &KeyInput) -> bool {
    (input.modifiers.is_empty() || input.modifiers == [Modifier::Shift])
        && input.key.is_key_string()
}
