use color_eyre::{eyre::Context, Result};
use futures::{channel::mpsc::UnboundedReceiver, stream::SplitSink, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;

use crate::core_proto::{ToBackend, ToFrontend};

pub struct ClientSendHandle(
    SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, tungstenite::Message>,
);

impl ClientSendHandle {
    #[tracing::instrument(skip(self))]
    pub async fn send_rpc(&mut self, call: ToFrontend) -> Result<(), tungstenite::Error> {
        tracing::debug!("Sending rpc call to client: {call:?}");
        let json = serde_json::to_string(&call).unwrap();
        self.0.send(tungstenite::Message::Text(json)).await
    }
}

pub async fn wait_for_client(
    addr: &str,
) -> Result<(ClientSendHandle, UnboundedReceiver<ToBackend>)> {
    let server_listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to start tcp server")?;

    // for now, we only accept a single client. This will need to be a loop later.
    let (stream, _) = server_listener.accept().await?;
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (ws_send, mut ws_recv) = ws_stream.split();

    let (mut to_backend_send, to_backend_recv) = futures::channel::mpsc::unbounded::<ToBackend>();

    tokio::spawn(async move {
        while let Some(msg) = ws_recv.next().await {
            match msg {
                Ok(tungstenite::Message::Text(json)) => {
                    match serde_json::from_str::<ToBackend>(&json) {
                        Ok(x) => {
                            if let Err(err) = to_backend_send.send(x).await {
                                tracing::warn!(
                                    "Stopping websocket receiver forwarding loop: {err}"
                                );
                                break;
                            }
                        },
                        Err(err) => tracing::error!("Error parsing rpc message: {err:?}"),
                    }
                },
                Ok(other) => tracing::warn!("Got unsupported websocket message: {other:?}"),
                Err(err) => tracing::error!("Got error message from websocket: {err:?}"),
            }
        }
    });

    Ok((ClientSendHandle(ws_send), to_backend_recv))
}
