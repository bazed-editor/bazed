use color_eyre::Result;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    bazed_core::app::start("127.0.0.1:6969").await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
