use std::{io::Write, path::PathBuf};

use bazed_rpc::core_proto::{CaretPosition, ToFrontend};
use uuid::Uuid;

use crate::buffer::Buffer;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, derive_more::Display, derive_more::Into)]
pub struct DocumentId(pub Uuid);

impl DocumentId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    pub fn gen() -> DocumentId {
        Self(Uuid::new_v4())
    }
}

pub struct Document {
    pub path: Option<PathBuf>,
    pub buffer: Buffer,
}

impl Document {
    pub fn open_ephemeral() -> Document {
        Self {
            path: None,
            buffer: Buffer::new_empty(),
        }
    }

    pub fn open_file(path: PathBuf) -> std::io::Result<Document> {
        let content = std::fs::read_to_string(&path)?;
        Ok(Self {
            path: Some(path),
            buffer: Buffer::new_from_string(content),
        })
    }

    /// Save the current buffer state to its path. Does nothing when no path is set.
    pub fn write_to_file(&self) -> std::io::Result<()> {
        let Some(ref path) = self.path else { return Ok(()) };
        let mut file = std::fs::File::options().write(true).open(path)?;
        file.write_all(self.buffer.content_to_string().as_bytes())
    }

    /// Create a notification for the frontend, that contains all relevant state of this document.
    ///
    /// *Note:* This will later be replaced with a proper
    /// damage tracking-style system that sends patches to the frontend.
    /// Additionally, this will later only send updates concerning
    /// the parts of the document that are currently visible / relevant in the frontend.
    pub fn create_update_notification(&self, id: DocumentId) -> ToFrontend {
        ToFrontend::UpdateDocument {
            document_id: id.0,
            text: self.buffer.content_to_string(),
            carets: self
                .buffer
                .all_carets()
                .map(|x| CaretPosition {
                    line: x.line,
                    col: x.col,
                })
                .into(),
        }
    }
}
