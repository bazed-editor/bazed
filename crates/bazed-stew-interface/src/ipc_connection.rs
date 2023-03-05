//! Stew connection channels ([`StewConnectionSender`], [`StewConnectionReceiver`]) based on
//! IPC through unnamed pipes, using
//! [interprocess::unnamed_pipe](https://docs.rs/interprocess/latest/interprocess/unnamed_pipe/index.html).

use blocking::Unblock;
use futures::{channel::mpsc::UnboundedSender, AsyncWriteExt, StreamExt};
use interprocess::unnamed_pipe::{UnnamedPipeReader, UnnamedPipeWriter};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{de::IoRead, StreamDeserializer};

use crate::stew_rpc::{self, StewConnectionReceiver, StewConnectionSender};

pub struct UnnamedPipeJsonWriter<T>(UnboundedSender<T>);

impl<T> Clone for UnnamedPipeJsonWriter<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> UnnamedPipeJsonWriter<T>
where
    T: Serialize + Send + Sync + 'static,
{
    pub fn new(writer: UnnamedPipeWriter) -> Self {
        let mut writer = Unblock::new(writer);
        let (send, mut recv) = futures::channel::mpsc::unbounded();
        tokio::spawn(async move {
            while let Some(value) = recv.next().await {
                if let Err(err) = writer.write_all(&serde_json::to_vec(&value).unwrap()).await {
                    tracing::error!("Error writing to pipe: {:?}. Stopping writer.", err);
                    break;
                }
            }
        });
        Self(send)
    }
}

#[async_trait::async_trait]
impl<T> StewConnectionSender<T> for UnnamedPipeJsonWriter<T>
where
    T: Serialize + Send + Sync + 'static,
{
    async fn send_to_stew(&mut self, msg: T) -> Result<(), stew_rpc::Error> {
        self.0
            .unbounded_send(msg)
            .map_err(|_| stew_rpc::Error::Connection("Connection closed".into()))?;
        Ok(())
    }
}

pub struct UnnamedPipeJsonReader<T>(
    Unblock<StreamDeserializer<'static, IoRead<UnnamedPipeReader>, T>>,
);

impl<T> UnnamedPipeJsonReader<T>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    pub fn new(reader: UnnamedPipeReader) -> Self {
        let deserializer = serde_json::Deserializer::from_reader(reader);
        Self(Unblock::new(deserializer.into_iter()))
    }
}

#[async_trait::async_trait]
impl<T> StewConnectionReceiver<T> for UnnamedPipeJsonReader<T>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    async fn recv_from_stew(&mut self) -> Result<Option<T>, stew_rpc::Error> {
        Ok(self.0.next().await.transpose()?)
    }
}
