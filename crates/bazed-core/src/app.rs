use std::sync::Arc;

use bazed_rpc::{core_proto::ToBackend, core_proto::ToFrontend, server::ClientSendHandle};
use color_eyre::Result;
use futures::StreamExt;
use tokio::sync::RwLock;

use crate::{document::Document, marked_rope::MarkerId};

pub struct App {
    document: Option<Document>,
    event_send: ClientSendHandle,
    cursors: Vec<MarkerId>,
}

impl App {
    #[tracing::instrument(skip(self))]
    async fn open_tmp(&mut self) -> Result<()> {
        let mut document = Document::open_tmp()?;
        let start_cursor = document.sticky_marker_at_start();
        self.event_send
            .send_rpc_notification(ToFrontend::Open {
                id: document.id().0.to_owned(),
                title: "<tmp>".to_string(),
                text: document.content_to_string(),
            })
            .await?;
        self.cursors.push(start_cursor);
        self.document = Some(document);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn open_file(&mut self, path: std::path::PathBuf) -> Result<()> {
        let mut document = Document::open_file(path.clone())?;
        let start_cursor = document.sticky_marker_at_start();
        self.event_send
            .send_rpc_notification(ToFrontend::Open {
                id: document.id().0.to_owned(),
                title: path.display().to_string(),
                text: document.content_to_string(),
            })
            .await?;
        self.cursors.push(start_cursor);
        self.document = Some(document);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn handle_rpc_call(&mut self, call: ToBackend) -> Result<()> {
        tracing::info!(call = ?call, "Handling rpc call");
        match call {
            ToBackend::KeyPressed(key) => {
                if let Some(c) = key.key.try_into_char() {
                    let Some(ref mut document) = self.document else { return Ok(()) };
                    for cursor in &self.cursors {
                        document.insert_char(*cursor, c)?;
                    }
                    self.event_send
                        .send_rpc_notification(ToFrontend::UpdateText {
                            id: document.id().0.to_owned(),
                            text: document.content_to_string(),
                        })
                        .await?;
                }
            },
            ToBackend::MouseInput { line, column } => {
                tracing::info!("mouse input: {column},{line}")
            },
        }
        Ok(())
    }
}

pub async fn start(addr: &str) -> Result<()> {
    let (send, mut recv) = bazed_rpc::server::wait_for_client(addr).await?;

    let core = Arc::new(RwLock::new(App {
        document: None,
        event_send: send,
        cursors: Vec::new(),
    }));

    tokio::spawn({
        let core = core.clone();
        async move {
            while let Some(rpc_call) = recv.next().await {
                let mut core = core.write().await;
                if let Err(err) = core.handle_rpc_call(rpc_call).await {
                    tracing::error!("Failed to handle rpc call: {err:?}");
                }
            }
        }
    });

    core.write().await.open_tmp().await?;

    Ok(())
}
