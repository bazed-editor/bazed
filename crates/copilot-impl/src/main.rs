use bazed_stew_interface::stew_rpc;
use copilot_interface::Copilot;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

struct Plugin;

#[async_trait::async_trait]
impl copilot_interface::Copilot for Plugin {
    async fn hello(&mut self, name: String) -> Result<String, stew_rpc::Error> {
        tracing::info!("from plugin: Hello, {name}!");
        Ok(format!("Hello, {name}!"))
    }
    async fn try_hello(&mut self, name: String) -> Result<Result<String, usize>, stew_rpc::Error> {
        tracing::info!("from plugin: Hello, {name}!");
        Ok(Ok(format!("Hello, {name}!")))
    }
}

#[tokio::main]
async fn main() {
    init_logging();
    tracing::info!("Copilot started");
    let plugin = Plugin;

    let mut client = bazed_stew_interface::init_client(plugin);
    tracing::info!("Stew client running");

    copilot_interface::server::initialize(&mut client)
        .await
        .unwrap();

    tracing::info!("Initialized");

    let mut copilot = copilot_interface::CopilotClientImpl::load(client.clone())
        .await
        .unwrap();

    let result = copilot.hello("foo".to_string()).await.unwrap();
    tracing::info!("Result hello: {result:?}");
    let result = copilot.try_hello("foo".to_string()).await.unwrap().unwrap();
    tracing::info!("Result try_hello: {result:?}");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

fn init_logging() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true).pretty();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
