use serde::{Deserialize, Serialize};

use crate::rpc_proto::{FunctionId, InvocationId, StewRpcCall, StewRpcMessage};

#[async_trait::async_trait]
pub trait StewConnection {
    type Error;
    async fn send<T: Serialize>(&self, msg: T) -> Result<(), Self::Error>;
    async fn recv<'a, T: Deserialize<'a>>(&self) -> Option<T>;
}

pub struct PluginServer<C> {
    conn: C,
}

impl<C: StewConnection> PluginServer<C> {
    pub fn new(conn: C) -> Self {
        Self { conn }
    }

    pub async fn call_fn_and_await_response(
        &self,
        fn_id: FunctionId,
        args: serde_json::Value,
        invocation_id: Option<InvocationId>,
    ) -> Result<(), C::Error> {
        self.conn
            .send(StewRpcCall::CallFunction {
                fn_id,
                args,
                invocation_id,
            })
            .await
    }

    pub async fn send_call(&self, msg: StewRpcCall) -> Result<(), C::Error> {
        self.conn.send(msg).await
    }

    pub async fn send_raw<T: Serialize>(&self, msg: T) -> Result<(), C::Error> {
        self.conn.send(msg).await
    }

    pub async fn recv_msg(&self) -> Option<StewRpcMessage> {
        self.conn.recv().await
    }

    pub async fn recv_raw<'a, T: Deserialize<'a>>(&self) -> Option<T> {
        self.conn.recv().await
    }
}
