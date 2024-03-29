use std::{collections::HashMap, sync::Arc};

use bazed_input_mapper::input_event::KeyInput;
use bazed_rpc::{
    core_proto::ToBackend,
    core_proto::{Coordinate, ToFrontend, ViewData},
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
            vim_interface: VimInterface::new(),
        }
    }

    async fn open_document(&mut self, document: Document) -> Result<()> {
        let document_id = DocumentId::gen();
        let view_id = ViewId::gen();
        let view = View::new(document_id, Viewport::new(0, 20));
        self.event_send
            .send_rpc(ToFrontend::OpenView {
                view_id: view_id.0,
                path: document.path.clone(),
                view_data: ViewData {
                    first_line: view.vp.first_line,
                    text: document.lines_in_viewport(&view.vp),
                    carets: document.caret_positions(),
                    vim_mode: self.vim_interface.mode.to_string(),
                },
            })
            .await?;
        self.documents.insert(document_id, document);
        self.views.insert(view_id, view);
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
                self.handle_key_pressed(ViewId(view_id), input).await?
            },

            ToBackend::MouseInput { view_id, position } => {
                self.handle_mouse_input(ViewId(view_id), position).await?
            },
            ToBackend::MouseScroll {
                view_id,
                line_delta,
            } => {
                self.handle_mouse_scroll(ViewId(view_id), line_delta)
                    .await?
            },
            ToBackend::ViewportChanged { view_id, height } => {
                self.handle_viewport_changed(ViewId(view_id), height)
                    .await?;
            },
        }
        Ok(())
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

    async fn handle_mouse_input(&mut self, view_id: ViewId, coords: Coordinate) -> Result<()> {
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

    pub fn views(&self) -> &HashMap<ViewId, View> {
        &self.views
    }
}

pub async fn start(addr: &str, path: Option<std::path::PathBuf>) -> Result<()> {
    loop {
        let path = path.clone();
        let (send, mut recv) = bazed_rpc::server::wait_for_client(addr).await?;

        let core = Arc::new(RwLock::new(App::new(send)));

        tokio::spawn({
            let core = core.clone();
            async move {
                let res = if let Some(path) = path {
                    core.write().await.open_file(path).await
                } else {
                    core.write().await.open_ephemeral().await
                };
                if let Err(err) = res {
                    tracing::error!(?err, "Error opening file");
                }

                while let Some(rpc_call) = recv.next().await {
                    let mut core = core.write().await;
                    if let Err(err) = core.handle_rpc_call(rpc_call).await {
                        tracing::error!("Failed to handle rpc call: {err:?}");
                    }
                }
            }
        });
    }
}
#[cfg(test)]
mod tests {
    use bazed_input_mapper::input_event::{Key, KeyInput, Modifiers, RawKey};
    use bazed_rpc::{
        core_proto::{ToBackend, ToFrontend},
        server::ClientSendHandle,
    };
    use futures::channel::mpsc::unbounded;

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

        // app_open_ephemeral should trigger a OpenView message
        app.open_ephemeral().await?;
        let view_id = expect_msg!("OpenDocument", to_frontend_recv, ToFrontend::OpenView { view_id, ..} => view_id);

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
            height: 150,
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
                modifiers: Modifiers::empty(),
                key: Key("A".to_string()),
                code: RawKey("KeyA".to_string()),
            },
        })
        .await?;
        expect_msg!("UpdateView", to_frontend_recv, ToFrontend::UpdateView { .. } => ());

        Ok(())
    }
}
