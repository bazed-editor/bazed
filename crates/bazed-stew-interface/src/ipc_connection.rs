use std::{io::Write, thread};

use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    StreamExt,
};
use interprocess::unnamed_pipe::{UnnamedPipeReader, UnnamedPipeWriter};
use serde::{de::DeserializeOwned, Serialize};

use crate::stew_rpc::{self, StewConnectionReceiver, StewConnectionSender};

#[derive(Clone)]
pub struct UnnamedPipeJsonWriter(UnboundedSender<serde_json::Value>);

impl UnnamedPipeJsonWriter {
    pub fn new(mut writer: UnnamedPipeWriter) -> Self {
        let (send, mut recv) = futures::channel::mpsc::unbounded();
        tokio::spawn(async move {
            while let Some(value) = recv.next().await {
                if let Err(err) = writer.write_all(&serde_json::to_vec(&value).unwrap()) {
                    tracing::error!("Error writing to pipe: {:?}. Stopping writer.", err);
                    break;
                }
            }
        });
        Self(send)
    }
}

#[async_trait::async_trait]
impl StewConnectionSender for UnnamedPipeJsonWriter {
    async fn send_to_stew<T: Serialize + Send + Sync + 'static>(
        &mut self,
        msg: T,
    ) -> Result<(), stew_rpc::Error> {
        self.0
            .unbounded_send(serde_json::to_value(msg).map_err(Into::<stew_rpc::Error>::into)?)
            .map_err(|_| stew_rpc::Error::Connection("Connection closed".into()))?;
        Ok(())
    }
}

pub struct UnnamedPipeJsonReader(UnboundedReceiver<Result<serde_json::Value, serde_json::Error>>);

impl UnnamedPipeJsonReader {
    pub fn new(reader: UnnamedPipeReader) -> Self {
        let (send, recv) = futures::channel::mpsc::unbounded();
        let deserializer = serde_json::Deserializer::from_reader(reader);
        thread::spawn(move || {
            for value in deserializer.into_iter() {
                if let Err(err) = send.unbounded_send(value) {
                    tracing::error!("Error sending to channel: {:?}. Stopping reader.", err);
                    break;
                }
            }
        });
        Self(recv)
    }
}

#[async_trait::async_trait]
impl StewConnectionReceiver for UnnamedPipeJsonReader {
    async fn recv_from_stew<T: DeserializeOwned + Send + Sync + 'static>(
        &mut self,
    ) -> Result<Option<T>, stew_rpc::Error> {
        Ok(self
            .0
            .next()
            .await
            .transpose()?
            .map(|x| match serde_json::from_value(x.clone()) {
                Ok(x) => Ok(x),
                Err(e) => {
                    tracing::error!(raw = ?x, "Error deserializing message: {e}");
                    Err(stew_rpc::Error::Serde(e))
                },
            })
            .transpose()?)
    }
}
