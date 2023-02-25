use std::sync::Arc;

use dashmap::DashMap;
use futures::{channel::oneshot, future::BoxFuture, Future};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::rpc_proto::{
    FunctionId, InvocationId, InvocationResponseData, PluginId, StewRpcCall, StewRpcMessage,
};

macro_rules! expect_invocation_result {
    ($value:expr, $p:pat => $e:expr $(,)?) => {
        match $value {
            $p => Ok($e),
            InvocationResponseData::InvocationFailed(err) => Err(Error::InvocationFailed(err)),
            other => Err(Error::UnexpectedInvocationResponse(
                serde_json::to_value(other).unwrap(),
            )),
        }
    };
}

#[async_trait::async_trait]
pub trait StewConnectionSender: Send + Sync + 'static {
    async fn send_to_stew<T: Serialize + Send + Sync>(&mut self, msg: T) -> Result<(), Error>;
}
#[async_trait::async_trait]
pub trait StewConnectionReceiver: Send + Sync + 'static {
    async fn recv_from_stew<T: DeserializeOwned>(&mut self) -> Result<Option<T>, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Connection(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    InvocationCanceled(#[from] oneshot::Canceled),
    #[error("The invocation failed: {0}")]
    InvocationFailed(serde_json::Value),
    #[error("Received a response to the invocation, but it was of an unexpected kind: {0}")]
    UnexpectedInvocationResponse(serde_json::Value),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

pub type PluginFn<D> = Box<
    dyn Fn(&mut D, serde_json::Value) -> BoxFuture<'static, Result<serde_json::Value, serde_json::Value>>
        + Send
        + Sync
        + 'static,
>;

pub struct StewClient<S, D> {
    stew_send: S,
    functions: Arc<DashMap<FunctionId, PluginFn<D>>>,
    invocations: Arc<DashMap<InvocationId, oneshot::Sender<InvocationResponseData>>>,
}

impl<S, D> StewClient<S, D>
where
    S: StewConnectionSender + Clone,
    D: Send + Sync + 'static,
{
    pub fn start<R: StewConnectionReceiver>(stew_send: S, mut stew_recv: R, mut userdata: D) -> Self {
        let invocations = Arc::new(DashMap::<_, oneshot::Sender<_>>::new());
        let functions = Arc::new(DashMap::new());
        tokio::spawn({
            let invocations = invocations.clone();
            let functions = functions.clone();
            let mut stew_send = stew_send.clone();
            async move {
                loop {
                    match stew_recv.recv_from_stew::<StewRpcMessage>().await {
                        Ok(Some(StewRpcMessage::FunctionCalled(call))) => {
                            let function = match functions.get(&call.internal_id) {
                                Some(f) => f,
                                None => {
                                    tracing::error!("Function not found");
                                    continue;
                                },
                            };
                            let function: &PluginFn<D> = &function;
                            let result = function(&mut userdata, call.args).await;
                            if let Some(invocation_id) = call.invocation_id {
                                let result = stew_send
                                    .send_to_stew(StewRpcCall::FunctionReturn {
                                        caller_id: call.caller_id,
                                        return_value: result.into(),
                                        invocation_id,
                                    })
                                    .await;
                                if let Err(result) = result {
                                    tracing::error!("{:?}", result);
                                }
                            }
                        },
                        Ok(Some(StewRpcMessage::InvocationResponse(response))) => {
                            if let Some(sender) = invocations.remove(&response.invocation_id) {
                                sender.1.send(response.kind).unwrap();
                            }
                        },
                        Err(err) => {
                            tracing::error!("{:?}", err);
                        },
                        Ok(None) => {
                            tracing::error!("Connection closed");
                            break;
                        },
                    }
                }
            }
        });
        Self {
            stew_send,
            functions,
            invocations,
        }
    }

    pub async fn load_plugin(
        &mut self,
        name: String,
        version_requirement: String,
    ) -> Result<PluginInfo, Error> {
        let invocation_id = InvocationId::gen();
        self.send_call(StewRpcCall::LoadPlugin {
            name,
            version_requirement,
            invocation_id,
        })
        .await?;
        expect_invocation_result!(
            self.await_invocation_result(invocation_id).await?,
            InvocationResponseData::PluginLoaded { plugin_id, version } => {
                PluginInfo { plugin_id, version }
            },
        )
    }

    pub async fn register_fn<F, Fut>(&mut self, name: String, function: F) -> Result<(), Error>
    where
        F: Fn(&mut D, serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<serde_json::Value, serde_json::Value>> + Send + 'static,
    {
        let function_id = FunctionId::gen();
        self.functions.insert(
            function_id,
            Box::new(move |userdata, args| Box::pin(function(userdata, args))),
        );
        self.send_call(StewRpcCall::RegisterFunction {
            fn_name: name,
            internal_id: function_id,
        })
        .await?;
        Ok(())
    }

    pub async fn get_fn(
        &mut self,
        plugin_id: PluginId,
        fn_name: String,
    ) -> Result<FunctionId, Error> {
        let invocation_id = InvocationId::gen();
        self.send_call(StewRpcCall::GetFunction {
            plugin_id,
            fn_name: fn_name.to_string(),
            invocation_id,
        })
        .await?;
        expect_invocation_result!(
            self.await_invocation_result(invocation_id).await?,
            InvocationResponseData::GotFunctionId(id) => id,
        )
    }

    pub async fn call_fn_ignore_response<T: Serialize>(
        &mut self,
        fn_id: FunctionId,
        args: T,
    ) -> Result<(), Error> {
        self.send_call(StewRpcCall::CallFunction {
            fn_id,
            args: serde_json::to_value(args).unwrap(),
            invocation_id: None,
        })
        .await?;
        Ok(())
    }

    pub async fn call_fn_and_await_response<O: DeserializeOwned, E: DeserializeOwned>(
        &mut self,
        fn_id: FunctionId,
        args: impl Serialize,
    ) -> Result<Result<O, E>, Error> {
        let invocation_id = InvocationId::gen();
        self.send_call(StewRpcCall::CallFunction {
            fn_id,
            args: serde_json::to_value(args).unwrap(),
            invocation_id: Some(invocation_id),
        })
        .await?;
        let result = expect_invocation_result!(
            self.await_invocation_result(invocation_id).await?,
            InvocationResponseData::FunctionReturned(result) => result,
        )?;
        Ok(result.parse_into_result()?)
    }

    async fn await_invocation_result(
        &self,
        invocation_id: InvocationId,
    ) -> Result<InvocationResponseData, Error> {
        let (send, recv) = oneshot::channel();
        self.invocations.insert(invocation_id, send);
        let result = recv.await?;
        self.invocations.remove(&invocation_id);
        Ok(result)
    }

    pub async fn notify_ready(&mut self) -> Result<(), Error> {
        self.stew_send
            .send_to_stew(StewRpcCall::PluginReady)
            .await
            .map_err(|x| Error::Connection(Box::new(x)))
    }

    pub async fn send_call(&mut self, msg: StewRpcCall) -> Result<(), Error> {
        self.stew_send
            .send_to_stew(msg)
            .await
            .map_err(|x| Error::Connection(Box::new(x)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginInfo {
    pub plugin_id: PluginId,
    pub version: String,
}
