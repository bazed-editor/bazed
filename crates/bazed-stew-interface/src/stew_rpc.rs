use std::sync::Arc;

use dashmap::DashMap;
use derivative::Derivative;
use dyn_clone::DynClone;
use futures::{channel::oneshot, future::BoxFuture};
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use tracing::Instrument;

use crate::rpc_proto::{
    FunctionCalled, FunctionId, InvocationId, InvocationResponseData, PluginId, StewRpcCall,
    StewRpcMessage,
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
pub trait StewConnectionSender<T>: DynClone + Send + Sync + 'static
where
    T: Serialize + Send + Sync + 'static,
{
    async fn send_to_stew(&mut self, msg: T) -> Result<(), Error>;
}

#[async_trait::async_trait]
pub trait StewConnectionReceiver<T>: Send + Sync + 'static
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    async fn recv_from_stew(&mut self) -> Result<Option<T>, Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Connection(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    InvocationCanceled(#[from] oneshot::Canceled),
    #[error("The invocation failed: {}", serde_json::to_string(&0).unwrap())]
    InvocationFailed(Value),
    #[error("Received a response to the invocation, but it was of an unexpected kind: {}", serde_json::to_string(&0).unwrap())]
    UnexpectedInvocationResponse(Value),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error("A function considered infallible returned an error anyways: {}", serde_json::to_string(&0).unwrap())]
    InfallibleFunctionFailed(serde_json::Value),
}

/// A method exposed by a plugin
type PluginFn<D> = Box<
    dyn for<'a> Fn(&'a mut D, Value) -> BoxFuture<'a, Result<Value, Value>> + Send + Sync + 'static,
>;

/// Base session type for a connection to the main stew system.
///
/// This can be cloned to get another handle to the same session.
pub struct StewSessionBase {
    stew_send: Box<dyn StewConnectionSender<StewRpcCall>>,
    invocations: Arc<DashMap<InvocationId, oneshot::Sender<InvocationResponseData>>>,
    function_call_recv: async_channel::Receiver<FunctionCalled>,
}

impl Clone for StewSessionBase {
    fn clone(&self) -> Self {
        Self {
            stew_send: dyn_clone::clone_box(&*self.stew_send),
            invocations: self.invocations.clone(),
            function_call_recv: self.function_call_recv.clone(),
        }
    }
}

/// A full stew session, wrapping the invocation and function-call event loop.
/// Registered functions will be stored as callbacks in the session,
/// and will be invoked when the corresponding [`StewRpcMessage::FunctionCalled`]-message is received.
///
/// This can be cloned to get another handle to the same session.
#[derive(derive_more::Deref, derive_more::DerefMut, Derivative)]
#[derivative(Clone)]
pub struct StewSession<D> {
    functions: Arc<DashMap<FunctionId, PluginFn<D>>>,
    #[deref]
    #[deref_mut]
    base: StewSessionBase,
}

impl<D> StewSession<D>
where
    D: Send + Sync + 'static,
{
    pub fn start(base: StewSessionBase, mut userdata: D) -> Self {
        let functions = Arc::new(DashMap::new());
        let mut stew_send = dyn_clone::clone_box(&*base.stew_send);
        let function_call_recv = base.function_call_recv.clone();
        tokio::spawn({
            let functions = functions.clone();
            async move {
                while let Ok(call) = function_call_recv.recv().await {
                    let Some(function) = functions.get(&call.internal_id) else {
                        tracing::error!("Function not found");
                        continue;
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
                }
            }
        });
        Self { base, functions }
    }

    pub async fn register_fn<F>(&mut self, name: &str, function: F) -> Result<(), Error>
    where
        F: for<'a> Fn(&'a mut D, Value) -> BoxFuture<'a, Result<Value, Value>>
            + Send
            + Sync
            + 'static,
    {
        let function_id = FunctionId::gen();
        self.functions.insert(
            function_id,
            Box::new(move |userdata: &mut D, args| Box::pin(function(userdata, args))),
        );
        self.send_call(StewRpcCall::RegisterFunction {
            fn_name: name.to_string(),
            internal_id: function_id,
        })
        .await?;
        Ok(())
    }
}

impl StewSessionBase {
    #[tracing::instrument(skip_all)]
    pub fn start<S, R>(stew_send: S, mut stew_recv: R) -> Self
    where
        S: StewConnectionSender<StewRpcCall>,
        R: StewConnectionReceiver<StewRpcMessage>,
    {
        let (function_call_send, function_call_recv) = async_channel::unbounded();
        let invocations = Arc::new(DashMap::<_, oneshot::Sender<_>>::new());
        tokio::spawn({
            let invocations = invocations.clone();
            async move {
                loop {
                    match stew_recv.recv_from_stew().await {
                        Ok(Some(StewRpcMessage::FunctionCalled(call))) => {
                            if let Err(err) = function_call_send.send(call).await {
                                tracing::error!("Failed to forward function call event: {err:?}");
                            }
                        },
                        Ok(Some(StewRpcMessage::InvocationResponse(response))) => {
                            if let Some(sender) = invocations.remove(&response.invocation_id) {
                                if let Err(err) = sender.1.send(response.kind) {
                                    tracing::error!("Failed to send invocation response: {err:?}");
                                }
                            }
                        },
                        Err(err) => {
                            tracing::error!("Received error from stew: {:?}", err);
                        },
                        Ok(None) => {
                            tracing::error!("Connection closed");
                            break;
                        },
                    }
                }
            }
            .in_current_span()
        });

        Self {
            stew_send: Box::new(stew_send),
            function_call_recv,
            invocations,
        }
    }

    pub async fn load_plugin(
        &mut self,
        name: String,
        version_requirement: VersionReq,
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

    #[tracing::instrument(skip(self, args))]
    pub async fn call_fn_ignore_response<T: Serialize>(
        &mut self,
        fn_id: FunctionId,
        args: T,
    ) -> Result<(), Error> {
        self.send_call_fn_call(fn_id, args, None).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, args))]
    pub async fn call_fn_and_await_response<O: DeserializeOwned, E: DeserializeOwned>(
        &mut self,
        fn_id: FunctionId,
        args: impl Serialize,
    ) -> Result<Result<O, E>, Error> {
        let invocation_id = InvocationId::gen();
        self.send_call_fn_call(fn_id, args, Some(invocation_id))
            .await?;
        let result = expect_invocation_result!(
            self.await_invocation_result(invocation_id).await?,
            InvocationResponseData::FunctionReturned(result) => result,
        )?;
        Ok(result.parse_into_result()?)
    }

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
    pub async fn send_call(&mut self, msg: StewRpcCall) -> Result<(), Error> {
        self.stew_send
            .send_to_stew(msg)
            .await
            .map_err(|x| Error::Connection(Box::new(x)))
    }

    async fn send_call_fn_call<T: Serialize>(
        &mut self,
        fn_id: FunctionId,
        args: T,
        invocation_id: Option<InvocationId>,
    ) -> Result<(), Error> {
        self.send_call(StewRpcCall::CallFunction {
            fn_id,
            args: serde_json::to_value(args).unwrap(),
            invocation_id,
        })
        .await
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginInfo {
    pub plugin_id: PluginId,
    pub version: Version,
}
