use copilot_interface::CopilotClient;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

struct Plugin {
    counter: usize,
}

#[async_trait::async_trait]
impl copilot_interface::Copilot for Plugin {
    //TODO make this not round-trip
    async fn plus(&mut self, n: usize) {
        self.counter += n;
    }
    async fn minus(&mut self, n: usize) -> Result<(), String> {
        if self.counter < n {
            Err("Can't subtract more than the current value".to_string())
        } else {
            self.counter -= n;
            Ok(())
        }
    }
    async fn value(&mut self) -> usize {
        self.counter
    }
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    init_logging();
    tracing::info!("Copilot started");
    let plugin = Plugin { counter: 0 };

    let mut stew_session = bazed_stew_interface::init_client(plugin);
    tracing::info!("Stew session running");

    copilot_interface::server::initialize(&mut stew_session).await?;
    tracing::info!("Initialized");

    let mut other_plugin: CopilotClient = CopilotClient::load(stew_session.clone()).await?;

    other_plugin.plus(5).await?;
    other_plugin.plus(5).await?;
    let result = other_plugin.value().await?;
    tracing::info!("Result value: {result:?}");
    let result = other_plugin.minus(15).await?;
    tracing::info!("Result minus: {result:?}");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

fn init_logging() {
    color_eyre::install().unwrap();
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
