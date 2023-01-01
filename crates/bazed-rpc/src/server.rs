use color_eyre::{eyre::Context, Result};
use futures::{
    channel::mpsc::{SendError, UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use tokio_tungstenite::tungstenite;

use crate::core_proto::{ToBackend, ToFrontend};

pub struct ClientSendHandle(pub UnboundedSender<ToFrontend>);

impl ClientSendHandle {
    #[tracing::instrument(skip(self))]
    pub async fn send_rpc(&mut self, call: ToFrontend) -> Result<(), SendError> {
        tracing::debug!("Sending rpc call to client: {call:?}");
        self.0.send(call).await
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
    let (mut ws_send, mut ws_recv) = ws_stream.split();

    let (mut to_backend_send, to_backend_recv) = futures::channel::mpsc::unbounded::<ToBackend>();
    let (to_frontend_send, mut to_frontend_recv) =
        futures::channel::mpsc::unbounded::<ToFrontend>();

    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = ws_recv.next() => {
                    match msg {
                        Some(Ok(tungstenite::Message::Text(json))) => {
                            match serde_json::from_str::<ToBackend>(&json) {
                                Ok(x) => {
                                    if let Err(err) = to_backend_send.send(x).await {
                                        tracing::warn!(
                                            "Stopping ToBackend receiver forwarding loop: {err}"
                                        );
                                        break;
                                    }
                                },
                                Err(err) => {
                                    tracing::error!(raw_json = json, "Error parsing rpc message: {err:?}")
                                },
                            }
                        },
                        Some(Ok(tungstenite::Message::Ping(bytes))) => {
                            if let Err(err) = ws_send.send(tungstenite::Message::Pong(bytes)).await {
                                tracing::error!("Error sending pong: {err}");
                                break;
                            }
                        },
                        Some(Ok(other)) => tracing::warn!("Got unsupported websocket message: {other:?}"),
                        Some(Err(err)) => tracing::error!("Got error message from websocket: {err:?}"),
                        None => {
                            break;
                        }
                    }
                }
                to_frontend_msg = to_frontend_recv.next() => {
                    tracing::debug!("Sending rpc call to client: {to_frontend_msg:?}");
                    let json = serde_json::to_string(&to_frontend_msg).unwrap();
                    if let Err(err) = ws_send.send(tungstenite::Message::Text(json)).await {
                        tracing::warn!("Stopping ToFrontend forwarding loop: {err}");
                        break;
                    }
                }
            };
        }
    });

    Ok((ClientSendHandle(to_frontend_send), to_backend_recv))
}
