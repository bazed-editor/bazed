use std::os::fd::FromRawFd;

use bazed_stew_interface::{
    ipc_connection::{UnnamedPipeJsonReader, UnnamedPipeJsonWriter},
    rpc_proto::{PluginId, PluginMetadata, StewRpcCall},
    stew_rpc::StewClient,
};
use interprocess::unnamed_pipe::{UnnamedPipeReader, UnnamedPipeWriter};
use serde_json::json;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main]
async fn main() {
    init_logging();
    tracing::info!("Copilot started");
    let writer = unsafe {
        UnnamedPipeWriter::from_raw_fd(std::env::args().nth(1).unwrap().parse().unwrap())
    };
    let writer = UnnamedPipeJsonWriter::new(writer);
    let reader = unsafe {
        UnnamedPipeReader::from_raw_fd(std::env::args().nth(2).unwrap().parse().unwrap())
    };
    let reader = UnnamedPipeJsonReader::new(reader);
    let _plugin_id: PluginId = PluginId(std::env::args().nth(3).unwrap().parse().unwrap());

    let mut client = StewClient::start(writer, reader, ());
    tracing::info!("Stew client running");

    client
        .send_call(StewRpcCall::Metadata(PluginMetadata {
            api_major: 1,
            api_minor: 0,
            name: "copilot".to_string(),
            version: "0.1.0".parse().unwrap(),
        }))
        .await
        .unwrap();
    tracing::info!("Sent metadata");

    client
        .register_fn("print", |_, args| async move {
            let args: String = serde_json::from_value(args).map_err(|e| json!(e.to_string()))?;
            tracing::info!("print: {args}");
            Ok(json!(format!("hello, {args}")))
        })
        .await
        .unwrap();

    let copilot = client
        .load_plugin("copilot".to_string(), "*".parse().unwrap())
        .await
        .unwrap();
    tracing::info!("copilot: {copilot:?}");
    let print_fn = client
        .get_fn(copilot.plugin_id, "print".to_string())
        .await
        .unwrap();
    tracing::info!("Got function: {print_fn}");

    let result: Result<String, serde_json::Value> = client
        .call_fn_and_await_response(print_fn, json!("foo"))
        .await
        .unwrap();
    tracing::info!("Result: {result:?}");

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
