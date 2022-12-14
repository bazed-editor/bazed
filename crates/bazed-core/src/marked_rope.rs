use std::collections::HashMap;

use ropey::Rope;
use uuid::Uuid;

/// A wrapper around the [ropey::Rope] datastructure that stores [Marker]s
/// together with the text. When the text changes, these markers move together with the text.
pub(crate) struct MarkedRope {
    rope: Rope,
    markers: HashMap<MarkerId, Marker>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Mark {} not found", .0)]
    UnknownMark(MarkerId),
}

impl MarkedRope {
    pub(crate) fn new(rope: Rope) -> Self {
        MarkedRope {
            rope,
            markers: HashMap::new(),
        }
    }

    pub(crate) fn insert_char(&mut self, m: MarkerId, c: char) -> Result<(), Error> {
        let marker = self.get_marker(m)?;
        self.rope.insert_char(marker.char_index, c);
        self.with_markers_after(marker.char_index, |x| x.char_index += 1);
        Ok(())
    }

    pub(crate) fn remove_char(&mut self, m: MarkerId) -> Result<(), Error> {
        let marker = self.get_marker(m)?;
        self.rope.remove(marker.char_index..marker.char_index);
        self.with_markers_after(marker.char_index, |x| x.char_index -= 1);
        Ok(())
    }

    fn with_markers_after(&mut self, char_index: usize, mut f: impl FnMut(&mut Marker)) {
        for marker in self.markers.values_mut() {
            if (marker.kind == MarkerKind::Sticky && marker.char_index <= char_index)
                || (marker.kind == MarkerKind::NonSticky || marker.char_index < char_index)
            {
                f(marker)
            }
        }
    }

    fn get_marker(&self, id: MarkerId) -> Result<Marker, Error> {
        self.markers
            .get(&id)
            .map(|x| *x)
            .ok_or_else(|| Error::UnknownMark(id))
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
pub struct MarkerId(Uuid);

impl std::fmt::Display for MarkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum MarkerKind {
    Sticky,
    NonSticky,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) struct Marker {
    kind: MarkerKind,
    char_index: usize,
}

impl PartialOrd for Marker {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.char_index.cmp(&other.char_index))
    }
}

impl Ord for Marker {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.char_index.cmp(&other.char_index)
    }
}

impl std::fmt::Display for Marker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char_index)
    }
}
