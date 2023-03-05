use bazed_stew_interface::{
    rpc_proto::StewRpcCall,
    stew_rpc::{self, StewClient, StewConnectionSender},
};

#[bazed_stew_macros::plugin(name = "copilot", version = "0.1")]
#[async_trait::async_trait]
pub trait Copilot {
    async fn hello(&mut self, name: String) -> Result<String, ()>;
}

// some-other-plugin-impl
pub async fn foo<S, D>(client: StewClient<S, D>)
where
    S: StewConnectionSender<StewRpcCall> + Clone + 'static,
    D: Send + Sync + 'static,
{
    let mut copilot = CopilotClientImpl::load(client).await.unwrap();
    copilot.hello("foo".to_string()).await.unwrap().unwrap();
}

// copilot-impl

pub struct CopilotImpl;

impl CopilotImpl {
    async fn initialize<S>(client: &mut StewClient<S, Self>) -> Result<(), stew_rpc::Error>
    where
        S: StewConnectionSender<StewRpcCall> + Clone + 'static,
    {
        server::register_functions(client).await
    }
}
#[async_trait::async_trait]
impl Copilot for CopilotImpl {
    async fn hello(&mut self, name: String) -> Result<Result<String, ()>, stew_rpc::Error> {
        Ok(Ok(format!("Hello, {name}!")))
    }
}
