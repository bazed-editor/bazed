use std::{
    ffi::OsString,
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

use bazed_rpc::core_proto::{CaretPosition, ToFrontend};
use uuid::Uuid;
use xi_rope::Rope;

use crate::{
    buffer::Buffer,
    view::{View, ViewId},
};

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

#[derive(Debug)]
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

    /// Asynchronously save the current buffer state to its path. Does nothing when no path is set.
    pub async fn write_to_file(&self) -> std::io::Result<()> {
        tracing::info!(document = ?self, "Saving document");
        if let Some(path) = self.path.clone() {
            let rope = self.buffer.head_rope().clone();
            tokio::task::spawn_blocking(move || write_rope_to_file(&path, &rope)).await??;
        }
        Ok(())
    }

    /// Create a notification for the frontend, that contains all relevant state of this document.
    ///
    /// *Note:* This will later be replaced with a proper
    /// damage tracking-style system that sends patches to the frontend.
    /// Additionally, this will later only send updates concerning
    /// the parts of the document that are currently visible / relevant in the frontend.
    pub fn create_update_notification(&self, view_id: ViewId, view: &View) -> ToFrontend {
        let lines = self
            .buffer
            .lines_between(view.vp.first_line, view.vp.last_line())
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();

        ToFrontend::UpdateView {
            view_id: view_id.into(),
            first_line: view.vp.first_line,
            height: view.vp.height,
            text: lines,
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

/// write a rope to a file by first writing to a .swp file and then renaming
fn write_rope_to_file(path: &std::path::Path, rope: &Rope) -> io::Result<()> {
    // we first write the text to a tmp file with the same name, but ending in .swp
    let tmp_extension = path.extension().map_or_else(
        || OsString::from("swp"),
        |ext| {
            let mut ext = ext.to_os_string();
            ext.push(".swp");
            ext
        },
    );
    let tmp_path = &path.with_extension(tmp_extension);
    let mut file = File::create(tmp_path)?;
    for chunk in rope.iter_chunks(..rope.len()) {
        file.write_all(chunk.as_bytes())?;
    }

    // remember the files permissions, if it already exists
    let permissions = std::fs::metadata(path).ok().map(|x| x.permissions());

    // rename the file to its actual desired name
    std::fs::rename(tmp_path, path)?;

    if let Some(permissions) = permissions {
        // And apply the permissions again
        if let Err(err) = std::fs::set_permissions(path, permissions) {
            tracing::warn!(
                "Failed to set permissions on file {}: {}",
                path.display(),
                err,
            )
        }
    }

    Ok(())
}
