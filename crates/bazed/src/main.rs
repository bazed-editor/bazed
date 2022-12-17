use color_eyre::Result;
use tracing::Level;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .pretty()
        .init();

    bazed_core::app::start("127.0.0.1:6969").await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
