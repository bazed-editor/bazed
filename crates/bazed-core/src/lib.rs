#![forbid(unreachable_pub)]
use std::{fs::File, io::BufReader, path::PathBuf};

use color_eyre::Result;
use marked_rope::{MarkedRope, MarkerId};
use ropey::Rope;

pub mod marked_rope;

pub struct Document {
    source: Option<PathBuf>,
    dirty: bool,
    content: MarkedRope,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            source: None,
            dirty: false,
            content: MarkedRope::new(Rope::default()),
        }
    }
}

impl Document {
    pub fn open(path: PathBuf) -> Result<Self> {
        let rope = MarkedRope::new(Rope::from_reader(BufReader::new(File::open(&path)?))?);
        Ok(Self {
            source: Some(path),
            dirty: false,
            content: rope,
        })
    }

    pub fn source(&self) -> Option<&PathBuf> {
        self.source.as_ref()
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn insert_char(&mut self, m: MarkerId, c: char) -> Result<(), marked_rope::Error> {
        self.content.insert_char(m, c)
    }

    pub fn delete_char(&mut self, m: MarkerId) -> Result<(), marked_rope::Error> {
        self.content.remove_char(m)
    }
}
