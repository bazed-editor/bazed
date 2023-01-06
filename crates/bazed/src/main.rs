use color_eyre::Result;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main(flavor = "multi_thread")]
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

    tokio::spawn(async {
        let edited_file = std::env::args().nth(1);

        bazed_core::app::start("127.0.0.1:6969", edited_file.map(Into::into))
            .await
            .unwrap();
    });

    bazed_tauri::run_frontend();
    Ok(())
}
