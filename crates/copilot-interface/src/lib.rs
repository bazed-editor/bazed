#[bazed_stew_macros::plugin(name = "copilot", version = "0.1.0", stew_version = "0.1")]
#[async_trait::async_trait]
pub trait Copilot {
    /// Add a value to the counter.
    async fn plus(&mut self, n: usize);
    /// Subtract a value from the counter. Fails if the counter would be < 0.
    async fn minus(&mut self, n: usize) -> Result<(), String>;
    /// Get the current value.
    async fn value(&mut self) -> usize;
}
