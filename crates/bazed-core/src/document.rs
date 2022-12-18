use bazed_rpc::core_proto::{CaretPosition, ToFrontend};
use uuid::Uuid;

use crate::buffer::Buffer;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, derive_more::Display, derive_more::Into)]
pub struct DocumentId(pub Uuid);
impl DocumentId {
    pub fn gen() -> DocumentId {
        DocumentId(Uuid::new_v4())
    }
}

pub struct Document {
    pub title: String,
    pub buffer: Buffer,
}

impl Document {
    pub fn open_ephemeral() -> Document {
        Self {
            title: "<unnamed>".to_string(),
            buffer: Buffer::open_ephemeral(),
        }
    }

    pub fn create_update_notification(&self, id: DocumentId) -> ToFrontend {
        ToFrontend::UpdateDocument {
            id: id.0,
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
