use example_plugin_interface::ExamplePluginClient;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

struct Plugin {
    counter: usize,
}

#[async_trait::async_trait]
impl example_plugin_interface::ExamplePlugin for Plugin {
    async fn increase(&mut self, n: usize) {
        self.counter += n;
    }

    async fn decrease(&mut self, n: usize) -> Result<(), String> {
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
    tracing::info!("Example plugin started");
    let plugin = Plugin { counter: 0 };
    let mut stew_session = bazed_stew_interface::init_session_with_state(plugin);
    tracing::info!("Stew session running");

    example_plugin_interface::server::initialize(&mut stew_session).await?;
    tracing::info!("Initialized");

    let mut other_plugin = ExamplePluginClient::load(stew_session.clone()).await?;

    other_plugin.increase(5).await?;
    other_plugin.increase(5).await?;
    let result = other_plugin.value().await?;
    assert_eq!(result, 10);
    tracing::info!("Result value: {result:?}");
    let result = other_plugin.decrease(15).await?;
    tracing::info!("Result minus: {result:?}");
    assert!(result.is_err());

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
