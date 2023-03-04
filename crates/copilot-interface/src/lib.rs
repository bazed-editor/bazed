use bazed_stew_interface::{
    rpc_proto::StewRpcCall,
    stew_rpc::{self, StewClient, StewConnectionSender},
};

#[bazed_stew_macros::plugin]
#[async_trait::async_trait]
pub trait Copilot {
    async fn hello(&mut self, name: String) -> Result<String, ()>;
}

// some-other-plugin-impl
pub async fn foo<S, D>(mut client: StewClient<S, D>)
where
    S: StewConnectionSender<StewRpcCall> + Clone + 'static,
    D: Send + Sync + 'static,
{
    let copilot = client
        .load_plugin("copilot".to_string(), "*".parse().unwrap())
        .await
        .unwrap();
    let mut copilot = CopilotClientImpl::initialize(client, copilot.plugin_id)
        .await
        .unwrap();
    copilot.hello("foo".to_string()).await.unwrap().unwrap();
}

// copilot-impl

pub struct CopilotImpl;

impl CopilotImpl {
    async fn initialize<S>(client: &mut StewClient<S, Self>) -> Result<(), stew_rpc::Error>
    where
        S: StewConnectionSender<StewRpcCall> + Clone + 'static,
    {
        register_functions(client).await
    }
}
#[async_trait::async_trait]
impl Copilot for CopilotImpl {
    async fn hello(&mut self, name: String) -> Result<Result<String, ()>, stew_rpc::Error> {
        Ok(Ok(format!("Hello, {name}!")))
    }
}
