use std::sync::Once;

use tracing::metadata::LevelFilter;

static INIT: Once = Once::new();
/// Set up logging and color_eyre for tests globally
pub(crate) fn setup_test() {
    INIT.call_once(|| {
        color_eyre::install().unwrap();
        std::env::set_var("RUST_BACKTRACE", "1");
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .without_time()
            .init();
    });
}
