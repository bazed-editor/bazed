#[bazed_stew_macros::plugin(name = "example-plugin", version = "0.1.0", stew_version = "0.1")]
#[async_trait::async_trait]
pub trait ExamplePlugin {
    /// Add a value to the counter.
    async fn increase(&mut self, n: usize);
    /// Subtract a value from the counter. Fails if the counter would be < 0.
    async fn decrease(&mut self, n: usize) -> Result<(), String>;
    /// Get the current value.
    async fn value(&mut self) -> usize;
}
