use color_eyre::{eyre::Context, Result};
use futures::{channel::mpsc::UnboundedReceiver, stream::SplitSink, SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite;

use crate::core_proto::{ToBackend, ToFrontend};

pub struct ClientSendHandle(
    SplitSink<tokio_tungstenite::WebSocketStream<TcpStream>, tungstenite::Message>,
);

impl ClientSendHandle {
    pub async fn send_rpc_notification(
        &mut self,
        notification: ToFrontend,
    ) -> Result<(), tungstenite::Error> {
        let json = serde_json::to_string(&notification).unwrap();
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
                                eprintln!("Stopping websocket receiver forwarding loop: {err}");
                                break;
                            }
                        },
                        Err(err) => eprintln!("Error parsing rpc message: {err:?}"),
                    }
                },
                Ok(other) => println!("Got unsupported websocket message: {other:?}"),
                Err(err) => eprintln!("Got error message from websocket: {err:?}"),
            }
        }
    });

    Ok((ClientSendHandle(ws_send), to_backend_recv))
}
