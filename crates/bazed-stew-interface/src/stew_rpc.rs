use std::sync::Arc;

use dashmap::DashMap;
use futures::{channel::oneshot, future::BoxFuture};
use semver::{Version, VersionReq};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

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
pub trait StewConnectionSender<T>: Send + Sync + 'static
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

pub type PluginFn<D> = Box<
    dyn for<'a> Fn(&'a mut D, Value) -> BoxFuture<'a, Result<Value, Value>> + Send + Sync + 'static,
>;

pub struct StewClient<S, D> {
    stew_send: S,
    functions: Arc<DashMap<FunctionId, PluginFn<D>>>,
    invocations: Arc<DashMap<InvocationId, oneshot::Sender<InvocationResponseData>>>,
}

impl<S: Clone, D> Clone for StewClient<S, D> {
    fn clone(&self) -> Self {
        Self {
            stew_send: self.stew_send.clone(),
            functions: self.functions.clone(),
            invocations: self.invocations.clone(),
        }
    }
}

impl<S, D> StewClient<S, D>
where
    S: StewConnectionSender<StewRpcCall> + Clone,
    D: Send + Sync + 'static,
{
    pub fn start<R: StewConnectionReceiver<StewRpcMessage>>(
        stew_send: S,
        mut stew_recv: R,
        mut userdata: D,
    ) -> Self {
        let invocations = Arc::new(DashMap::<_, oneshot::Sender<_>>::new());
        let functions = Arc::new(DashMap::new());
        tokio::spawn({
            let invocations = invocations.clone();
            let functions = functions.clone();
            let mut stew_send = stew_send.clone();
            async move {
                loop {
                    match stew_recv.recv_from_stew().await {
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
                                if let Err(err) = sender.1.send(response.kind) {
                                    tracing::error!("Failed to send invocation response: {err:?}");
                                }
                            }
                        },
                        Err(err) => {
                            tracing::error!("serde error: {:?}", err);
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

    pub async fn call_fn_and_await_response_infallible<O: DeserializeOwned>(
        &mut self,
        fn_id: FunctionId,
        args: impl Serialize,
    ) -> Result<O, Error> {
        match self.call_fn_and_await_response(fn_id, args).await? {
            Ok(result) => Ok(result),
            Err(err) => Err(Error::InfallibleFunctionFailed(err)),
        }
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
    pub version: Version,
}
