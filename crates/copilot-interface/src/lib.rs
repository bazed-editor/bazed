use bazed_stew_interface::{
    rpc_proto::StewRpcCall,
    stew_rpc::{StewClient, StewConnectionSender},
};

#[bazed_stew_macros::plugin(name = "copilot", version = "0.1.0", stew_version = "0.1")]
#[async_trait::async_trait]
pub trait Copilot {
    async fn hello(&mut self, name: String) -> String;
    async fn try_hello(&mut self, name: String) -> Result<String, usize>;
}

// some-other-plugin-impl
pub async fn foo<S, D>(client: StewClient<S, D>)
where
    S: StewConnectionSender<StewRpcCall> + Clone + 'static,
    D: Send + Sync + 'static,
{
    let mut copilot = CopilotClientImpl::load(client).await.unwrap();
    copilot.hello("foo".to_string()).await.unwrap();
}
