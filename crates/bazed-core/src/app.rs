use std::{collections::HashMap, sync::Arc};

use bazed_rpc::{
    core_proto::ToBackend,
    core_proto::{CaretPosition, ToFrontend},
    keycode::KeyInput,
    server::ClientSendHandle,
};
use color_eyre::Result;
use futures::StreamExt;
use tokio::sync::RwLock;

use crate::{
    buffer::position::Position,
    document::{Document, DocumentId},
    view::{View, ViewId, Viewport},
    vim_interface::VimInterface,
};

const SCROLL_OFF: usize = 3;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("No document with id {0} found")]
    InvalidDocumentId(DocumentId),
    #[error("No view with id {0} found")]
    InvalidViewId(ViewId),
}

pub struct App {
    documents: HashMap<DocumentId, Document>,
    views: HashMap<ViewId, View>,
    event_send: ClientSendHandle,
    vim_interface: VimInterface,
}

impl App {
    pub fn new(event_send: ClientSendHandle) -> Self {
        App {
            documents: HashMap::new(),
            event_send,
            views: HashMap::new(),
            vim_interface: VimInterface::default(),
        }
    }

    async fn open_document(&mut self, document: Document) -> Result<()> {
        let id = DocumentId::gen();
        self.event_send
            .send_rpc(ToFrontend::OpenDocument {
                document_id: id.0,
                path: document.path.clone(),
                text: document.buffer.content_to_string(),
            })
            .await?;
        self.documents.insert(id, document);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn open_file(&mut self, path: std::path::PathBuf) -> Result<()> {
        let document = Document::open_file(path)?;
        self.open_document(document).await
    }

    #[tracing::instrument(skip(self))]
    async fn open_ephemeral(&mut self) -> Result<()> {
        let document = Document::open_ephemeral();
        self.open_document(document).await
    }

    #[tracing::instrument(skip(self))]
    async fn handle_rpc_call(&mut self, call: ToBackend) -> Result<()> {
        tracing::info!(call = ?call, "Handling rpc call");
        match call {
            ToBackend::KeyPressed { view_id, input } => {
                self.handle_key_pressed(ViewId::from_uuid(view_id), input)
                    .await?
            },

            ToBackend::MouseInput { view_id, position } => {
                self.handle_mouse_input(ViewId::from_uuid(view_id), position)
                    .await?
            },
            ToBackend::MouseScroll {
                view_id,
                line_delta,
            } => {
                self.handle_mouse_scroll(ViewId::from_uuid(view_id), line_delta)
                    .await?
            },

            ToBackend::ViewportChanged { view_id, height } => {
                self.handle_viewport_changed(ViewId::from_uuid(view_id), height)
                    .await?;
            },
            ToBackend::ViewOpened {
                request_id,
                document_id,
                height,
            } => {
                let view_id = self
                    .handle_view_opened(DocumentId::from_uuid(document_id), height)
                    .await?;
                self.event_send
                    .send_rpc(ToFrontend::ViewOpenedResponse {
                        request_id,
                        view_id: view_id.into(),
                    })
                    .await?;
            },
            ToBackend::SaveDocument { document_id } => {
                self.handle_save_document(DocumentId::from_uuid(document_id))
                    .await?;
            },
        }
        Ok(())
    }

    async fn handle_save_document(&mut self, document_id: DocumentId) -> Result<()> {
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(Error::InvalidDocumentId(document_id))?;
        Ok(document.write_to_file().await?)
    }

    async fn handle_viewport_changed(&mut self, view_id: ViewId, height: usize) -> Result<()> {
        let view = self
            .views
            .get_mut(&view_id)
            .ok_or(Error::InvalidViewId(view_id))?;
        let needs_new_view_info = height > view.vp.height;
        view.vp.height = height;

        if needs_new_view_info {
            let document = self
                .documents
                .get(&view.document_id)
                .ok_or(Error::InvalidDocumentId(view.document_id))?;
            self.event_send
                .send_rpc(document.create_update_notification(
                    view_id,
                    view,
                    self.vim_interface.mode,
                ))
                .await?;
        }
        Ok(())
    }

    async fn handle_key_pressed(&mut self, view_id: ViewId, input: KeyInput) -> Result<()> {
        let view = self
            .views
            .get_mut(&view_id)
            .ok_or(Error::InvalidViewId(view_id))?;
        let document = self
            .documents
            .get_mut(&view.document_id)
            .ok_or(Error::InvalidDocumentId(view.document_id))?;

        self.vim_interface
            .on_input(view, &mut document.buffer, input);

        // Make sure to keep the cursor on screen
        let caret_line = document.buffer.primary_caret_position().line;
        view.vp = view.vp.with_line_in_view(caret_line, SCROLL_OFF);

        self.event_send
            .send_rpc(document.create_update_notification(view_id, view, self.vim_interface.mode))
            .await?;
        Ok(())
    }

    async fn handle_mouse_input(&mut self, view_id: ViewId, coords: CaretPosition) -> Result<()> {
        let view = self
            .views
            .get_mut(&view_id)
            .ok_or(Error::InvalidViewId(view_id))?;
        let document = self
            .documents
            .get_mut(&view.document_id)
            .ok_or(Error::InvalidDocumentId(view.document_id))?;
        document
            .buffer
            .jump_caret_to_position(Position::new(coords.line, coords.col), false);
        self.event_send
            .send_rpc(document.create_update_notification(view_id, view, self.vim_interface.mode))
            .await?;
        Ok(())
    }

    async fn handle_mouse_scroll(&mut self, view_id: ViewId, line_delta: i32) -> Result<()> {
        let mut view = self
            .views
            .get_mut(&view_id)
            .ok_or(Error::InvalidViewId(view_id))?;

        let document = self
            .documents
            .get(&view.document_id)
            .ok_or(Error::InvalidDocumentId(view.document_id))?;

        let line_count = document.buffer.line_count();

        view.vp.first_line = usize::min(
            view.vp
                .first_line
                .saturating_add_signed(line_delta as isize),
            line_count,
        );

        self.event_send
            .send_rpc(document.create_update_notification(view_id, view, self.vim_interface.mode))
            .await?;

        Ok(())
    }

    async fn handle_view_opened(
        &mut self,
        document_id: DocumentId,
        height: usize,
    ) -> Result<ViewId> {
        if !self.documents.contains_key(&document_id) {
            return Err(Error::InvalidDocumentId(document_id).into());
        }
        let view = View::new(document_id, Viewport::new(0, height));
        let id = ViewId::gen();
        self.views.insert(id, view);
        Ok(id)
    }

    pub fn views(&self) -> &HashMap<ViewId, View> {
        &self.views
    }
}

pub async fn start(addr: &str, path: Option<std::path::PathBuf>) -> Result<()> {
    let (send, mut recv) = bazed_rpc::server::wait_for_client(addr).await?;

    let core = Arc::new(RwLock::new(App::new(send)));

    if let Some(path) = path {
        core.write().await.open_file(path).await?;
    } else {
        core.write().await.open_ephemeral().await?;
    }

    while let Some(rpc_call) = recv.next().await {
        let mut core = core.write().await;
        if let Err(err) = core.handle_rpc_call(rpc_call).await {
            tracing::error!("Failed to handle rpc call: {err:?}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use bazed_rpc::{
        core_proto::{RequestId, ToBackend, ToFrontend},
        keycode::{Key, KeyInput},
        server::ClientSendHandle,
    };
    use futures::channel::mpsc::unbounded;
    use uuid::Uuid;

    use super::App;
    use crate::test_util;

    macro_rules! expect_msg {
        ($s:literal, $recv:ident, $p:pat => $e:expr) => {
            match $recv.try_next()?.unwrap() {
                $p => $e,
                _ => panic!(std::concat!("Expected ", $s)),
            }
        };
    }

    /// Set up a view to run tests against
    async fn setup_view() -> color_eyre::Result<(
        App,
        futures::channel::mpsc::UnboundedReceiver<ToFrontend>,
        uuid::Uuid,
    )> {
        let (to_frontend_send, mut to_frontend_recv) = unbounded::<ToFrontend>();
        let mut app = App::new(ClientSendHandle(to_frontend_send));

        // app_open_ephemeral should trigger an OpenDocument response
        app.open_ephemeral().await?;
        let document_id = expect_msg!("OpenDocument", to_frontend_recv, ToFrontend::OpenDocument { document_id, ..} => document_id);

        // ViewOpened should trigger a ViewOpenedResponse response
        app.handle_rpc_call(ToBackend::ViewOpened {
            request_id: RequestId(Uuid::new_v4()),
            document_id,
            height: 10,
        })
        .await?;
        let view_id = expect_msg!("ViewOpenedResponse", to_frontend_recv, ToFrontend::ViewOpenedResponse { view_id, .. } => view_id);

        Ok((app, to_frontend_recv, view_id))
    }

    #[tokio::test]
    async fn test_setup_view() -> color_eyre::Result<()> {
        setup_view().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_viewport_changed() -> color_eyre::Result<()> {
        test_util::setup_test();
        let (mut app, mut to_frontend_recv, view_id) = setup_view().await?;
        // Expanding the Viewport should trigger an UpdateView response
        app.handle_rpc_call(ToBackend::ViewportChanged {
            view_id,
            height: 15,
        })
        .await?;
        expect_msg!("UpdateView", to_frontend_recv, ToFrontend::UpdateView { .. } => {});

        // Shrinking the Viewport should not trigger an UpdateView response
        app.handle_rpc_call(ToBackend::ViewportChanged { view_id, height: 5 })
            .await?;
        // Panic if there is a message
        to_frontend_recv.try_next().unwrap_err();
        Ok(())
    }

    #[tokio::test]
    async fn test_keypress() -> color_eyre::Result<()> {
        let (mut app, mut to_frontend_recv, view_id) = setup_view().await?;

        app.handle_rpc_call(ToBackend::KeyPressed {
            view_id,
            input: KeyInput {
                modifiers: vec![],
                key: Key::Char('A'),
            },
        })
        .await?;
        expect_msg!("UpdateView", to_frontend_recv, ToFrontend::UpdateView { .. } => ());

        Ok(())
    }
}
