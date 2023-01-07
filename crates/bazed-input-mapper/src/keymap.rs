use std::collections::HashMap;

use crate::input_event::{KeyInput, Modifier};

#[derive(Debug)]
pub struct Keymap<V> {
    pub map: HashMap<KeyInput, KeymapNode<V>>,
    /// If no other map was matched, but the pressed key is a printable character
    /// (corresponding to <https://www.w3.org/TR/uievents-key/#key-string>)
    /// this node will be matched
    pub on_any_printable: Option<KeymapNode<V>>,
}

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
        map: HashMap<KeyInput, KeymapNode<V>>,
        on_any_printable: Option<KeymapNode<V>>,
    ) -> Self {
        Self {
            map,
            on_any_printable,
        }
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

    pub fn new_from_map(map: HashMap<KeyInput, KeymapNode<V>>) -> Self {
        Self::new(map, None)
    }

    pub fn descriptions(&self) -> impl Iterator<Item = (&KeyInput, &str)> {
        self.map.iter().map(|(k, v)| (k, v.description()))
    }

    pub fn map<O>(self, f: &dyn Fn(V) -> O) -> Keymap<O> {
        let map = self.map.into_iter().map(|(k, v)| (k, v.map(&f))).collect();
        let on_any_printable = self.on_any_printable.map(|v| v.map(f));
        Keymap {
            map,
            on_any_printable,
        }
    }

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
