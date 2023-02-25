use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    SinkExt, StreamExt,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;

use crate::stew_rpc::{self, StewClient, StewConnectionReceiver, StewConnectionSender};

/*
stew_plugin! {
    api_version = "1.1",
    version = b"1.1.0\0",
    name = b"I am an epic plugin\0",
    init = init,
    main = main,
    imports = [
        foo::bar: fn(x: *const ::std::ffi::c_char) -> usize,
        foo::baz: fn(x: *const ::std::ffi::c_char) -> usize,
    ]
}
*/

#[async_trait::async_trait]
impl StewConnectionSender for UnboundedSender<String> {
    async fn send_to_stew<T: Serialize + Send + Sync>(
        &mut self,
        message: T,
    ) -> Result<(), stew_rpc::Error> {
        self.send(serde_json::to_string(&message)?)
            .await
            .map_err(|_| stew_rpc::Error::Connection("Connection closed".into()))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl StewConnectionReceiver for UnboundedReceiver<String> {
    async fn recv_from_stew<T: DeserializeOwned>(&mut self) -> Result<Option<T>, stew_rpc::Error> {
        Ok(self
            .next()
            .await
            .map(|x| serde_json::from_str(&x))
            .transpose()?)
    }
}

#[allow(unused)]
async fn demo() -> Result<(), stew_rpc::Error> {
    let (stew_send, stew_recv) = futures::channel::mpsc::unbounded();
    let (plugin_send, plugin_recv) = futures::channel::mpsc::unbounded();
    let mut stew_client = StewClient::start(stew_send, plugin_recv, ());
    let banana = stew_client
        .load_plugin("banana".to_string(), ">2.5".parse().unwrap())
        .await?;

    let banana_applepie = stew_client
        .get_fn(banana.plugin_id, "applepie".to_string())
        .await?;

    stew_client
        .register_fn("tractor", |(), args| async move {
            let speed: usize = args["speed"].as_u64().unwrap() as usize;
            let color: String = args["color"].to_string();
            Ok(json!(format!("{color} tractor with speed {speed}")))
        })
        .await?;

    stew_client.notify_ready().await?;

    let mut iv = tokio::time::interval(tokio::time::Duration::from_secs(1));
    loop {
        iv.tick().await;
        let fn_result = stew_client
            .call_fn_and_await_response::<bool, String>(
                banana_applepie,
                serde_json::json!({"s": "hello", "n": 12}),
            )
            .await?;
        match fn_result {
            Ok(x) => println!("banana applepie returned {x}"),
            Err(e) => println!("banana applepie errored: {e}"),
        }
    }
}
