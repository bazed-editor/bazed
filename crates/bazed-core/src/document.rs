use std::{fs::File, io::BufReader, path::PathBuf};

use color_eyre::Result;
use ropey::Rope;
use uuid::Uuid;

use crate::marked_rope::{MarkedRope, MarkerId};

pub struct DocumentId(pub Uuid);
impl DocumentId {
    pub fn gen() -> DocumentId {
        DocumentId(Uuid::new_v4())
    }
}

pub struct Document {
    id: DocumentId,
    file: Option<PathBuf>,
    dirty: bool,
    content: MarkedRope,
}

impl Document {
    pub fn open_tmp() -> Result<Self> {
        let rope = MarkedRope::new(Rope::new());
        Ok(Self {
            id: DocumentId::gen(),
            file: None,
            dirty: false,
            content: rope,
        })
    }

    pub fn open_file(path: PathBuf) -> Result<Self> {
        let rope = MarkedRope::new(Rope::from_reader(BufReader::new(File::open(&path)?))?);
        Ok(Self {
            id: DocumentId::gen(),
            file: Some(path),
            dirty: false,
            content: rope,
        })
    }

    pub fn file(&self) -> Option<&PathBuf> {
        self.file.as_ref()
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn insert_char(&mut self, m: MarkerId, c: char) -> Result<(), crate::marked_rope::Error> {
        let res = self.content.insert_char(m, c);
        if res.is_ok() {
            self.dirty = true;
        }
        res
    }

    pub fn delete_char(&mut self, m: MarkerId) -> Result<(), crate::marked_rope::Error> {
        let res = self.content.remove_char(m);
        if res.is_ok() {
            self.dirty = true;
        }
        res
    }

    pub fn sticky_marker_at_start(&mut self) -> MarkerId {
        self.content.sticky_marker_at_start()
    }

    pub fn id(&self) -> &DocumentId {
        &self.id
    }

    pub fn content_to_string(&self) -> String {
        self.content.rope().to_string()
    }
}
