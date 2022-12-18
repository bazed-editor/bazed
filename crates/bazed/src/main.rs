use color_eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(false).pretty();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    let edited_file = std::env::args().nth(1);

    bazed_core::app::start("127.0.0.1:6969", edited_file.map(Into::into)).await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
