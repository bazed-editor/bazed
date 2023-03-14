#![deny(unreachable_pub)]

use std::{
    collections::HashMap, os::fd::AsRawFd, path::PathBuf, process::Command, sync::Arc, thread,
};

use bazed_stew_interface::rpc_proto::{
    FunctionCalled, FunctionId, InvocationId, InvocationResponse, InvocationResponseData, PluginId,
    PluginMetadata, StewRpcCall, StewRpcMessage,
};
use dashmap::DashMap;
use executable::search_plugin;
use futures::{channel::mpsc::UnboundedSender, StreamExt};
use semver::{Version, VersionReq};
use serde::Serialize;
use serde_json::{de::IoRead, json};
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod executable;

// TODO ensure that loaded plugins names and version match their sent metadata

pub async fn run_stew(load_path: Vec<PathBuf>) {
    let (rpc_call_send, mut rpc_call_recv) = futures::channel::mpsc::unbounded();
    let mut stew = Stew {
        load_path: load_path.clone(),
        plugins: Arc::new(DashMap::new()),
        rpc_call_send,
    };
    let example_plugin = search_plugin(&load_path, "example-plugin", &"*".parse().unwrap());
    if let Some(example_plugin) = example_plugin {
        stew.start_plugin(&example_plugin).await;
    } else {
        tracing::error!("Failed to find example plugin");
    }
    tracing::info!("Starting to listen to rpc calls");

    while let Some((caller_id, call)) = rpc_call_recv.next().await {
        stew.handle_rpc_call(caller_id, call).await;
    }
    tracing::info!("Stew exiting...");
}

pub struct Stew {
    load_path: Vec<PathBuf>,
    plugins: Arc<DashMap<PluginId, RwLock<PluginState>>>,
    rpc_call_send: UnboundedSender<(PluginId, StewRpcCall)>,
}

impl Stew {
    #[tracing::instrument(skip(self))]
    pub async fn start_plugin(&mut self, plugin: &executable::PluginExecutable) -> PluginId {
        tracing::info!("Starting plugin: {plugin}");

        let (to_stew_write, to_stew_read) = interprocess::unnamed_pipe::pipe().unwrap();
        let (to_plugin_write, to_plugin_read) = interprocess::unnamed_pipe::pipe().unwrap();
        let plugin_id = PluginId(Uuid::new_v4());
        Command::new(&plugin.path)
            .arg(to_stew_write.as_raw_fd().to_string())
            .arg(to_plugin_read.as_raw_fd().to_string())
            .arg(plugin_id.0.to_string())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("Failed to start plugin");
        tracing::info!("Started plugin with id {plugin_id}");

        let rpc_call_send = self.rpc_call_send.clone();
        let plugins = self.plugins.clone();
        thread::spawn(move || {
            let mut stream =
                serde_json::StreamDeserializer::<_, StewRpcCall>::new(IoRead::new(to_stew_read));

            let metadata = loop {
                match stream.next() {
                    Some(Ok(StewRpcCall::Metadata(meta))) => break meta,
                    Some(Ok(other)) => {
                        tracing::warn!("Discarding non-metadata rpc call: {other:?}");
                    },
                    Some(Err(err)) => {
                        tracing::error!("Failed parsing RPC call: {err}");
                    },
                    None => {
                        return;
                    },
                }
            };
            tracing::info!("Got metadata: {metadata:?}");

            let plugin = PluginState::new(plugin_id, metadata, to_plugin_write);

            plugins.insert(plugin_id, RwLock::new(plugin));

            for item in stream {
                match item {
                    Ok(item) => {
                        if rpc_call_send.unbounded_send((plugin_id, item)).is_err() {
                            break;
                        }
                    },
                    Err(err) => {
                        tracing::error!("Failed parsing RPC call: {err}");
                    },
                }
            }
        });
        plugin_id
    }

    #[tracing::instrument(skip(self), fields(%caller_id, ?call))]
    pub async fn handle_rpc_call(&mut self, caller_id: PluginId, call: StewRpcCall) {
        tracing::trace!("Handling RPC call");
        match call {
            StewRpcCall::RegisterFunction {
                fn_name,
                internal_id,
            } => {
                let Some(caller) = self.plugins.get(&caller_id) else {
                    tracing::error!("Caller {caller_id} not found");
                    return
                };
                let mut caller = caller.write().await;
                let fn_id = FunctionId::gen();
                caller.function_names.insert(fn_name, fn_id);
                caller.internal_function_id.insert(fn_id, internal_id);
            },
            StewRpcCall::GetFunction {
                plugin_id,
                fn_name,
                invocation_id,
            } => {
                let fn_id = {
                    let Some(plugin) = self.plugins.get(&plugin_id) else {
                        self.send_invocation_failure_to(caller_id, invocation_id, "Plugin not found")
                            .await;
                        return;
                    };
                    let plugin = plugin.read().await;
                    let Some(fn_id) = plugin.function_names.get(&fn_name) else {
                        self.send_invocation_failure_to(caller_id, invocation_id, "Function not found")
                            .await;
                        return;
                    };
                    *fn_id
                };
                self.send_response_to(
                    caller_id,
                    invocation_id,
                    InvocationResponseData::GotFunctionId(fn_id),
                )
                .await;
            },
            StewRpcCall::CallFunction {
                fn_id,
                args,
                invocation_id,
            } => {
                let Some(caller) = self.plugins.get(&caller_id) else {
                    tracing::error!("Caller {caller_id} not found");
                    return;
                };
                let mut caller = caller.write().await;
                let Some(&internal_id) = caller.internal_function_id.get(&fn_id) else {
                    if let Some(invocation_id) = invocation_id {
                        if let Err(err) = caller.send_response(
                            invocation_id,
                            InvocationResponseData::InvocationFailed(json!("Function not found"))
                        ) {
                            tracing::error!("Failed sending invocation failed message: {err}");
                        }
                    }
                    return;
                };
                let result = caller.send_function_called(FunctionCalled {
                    internal_id,
                    args,
                    invocation_id,
                    caller_id,
                });
                if let Err(err) = result {
                    tracing::error!("Failed sending function called message: {err}");
                }
            },
            StewRpcCall::FunctionReturn {
                caller_id: original_caller_id,
                return_value,
                invocation_id,
            } => {
                self.send_response_to(
                    original_caller_id,
                    invocation_id,
                    InvocationResponseData::FunctionReturned(return_value),
                )
                .await
            },
            StewRpcCall::LoadPlugin {
                name,
                version_requirement,
                invocation_id,
            } => {
                if let Some((plugin_id, version)) =
                    self.find_plugin_data(&name, &version_requirement).await
                {
                    tracing::info!("Found already loaded plugin");
                    self.send_response_to(
                        caller_id,
                        invocation_id,
                        InvocationResponseData::PluginLoaded { plugin_id, version },
                    )
                    .await;
                } else if let Some(found) =
                    search_plugin(&self.load_path, &name, &version_requirement)
                {
                    let plugin_id = self.start_plugin(&found).await;
                    self.send_response_to(
                        caller_id,
                        invocation_id,
                        InvocationResponseData::PluginLoaded {
                            plugin_id,
                            version: found.version,
                        },
                    )
                    .await;
                } else {
                    tracing::warn!("Failed to find plugin");
                    self.send_invocation_failure_to(caller_id, invocation_id, "Plugin not found")
                        .await;
                }
            },
            StewRpcCall::Metadata(_) => {
                tracing::warn!("Discarding metadata rpc call");
            },
            StewRpcCall::PluginReady => {
                tracing::warn!("Discarding plugin ready rpc call");
            },
        }
    }

    async fn find_plugin_data(
        &self,
        name: &str,
        version_req: &VersionReq,
    ) -> Option<(PluginId, Version)> {
        for plugin in self.plugins.iter() {
            let plugin = plugin.value().read().await;
            if plugin.metadata.name == name && version_req.matches(&plugin.metadata.version) {
                return Some((plugin.id, plugin.metadata.version.clone()));
            }
        }
        None
    }

    async fn send_invocation_failure_to<T: Serialize>(
        &self,
        plugin_id: PluginId,
        invocation_id: InvocationId,
        data: T,
    ) {
        self.send_response_to(
            plugin_id,
            invocation_id,
            InvocationResponseData::InvocationFailed(serde_json::to_value(data).unwrap()),
        )
        .await;
    }

    async fn send_response_to(
        &self,
        plugin_id: PluginId,
        invocation_id: InvocationId,
        data: InvocationResponseData,
    ) {
        let Some(caller) = self.plugins.get(&plugin_id) else {
            tracing::error!("Caller {plugin_id} not found");
            return
        };
        let result = caller.write().await.send_response(invocation_id, data);
        if let Err(err) = result {
            tracing::error!("Failed to send response to {plugin_id}: {err}");
        }
    }
}

pub struct PluginState {
    pub id: PluginId,
    pub metadata: PluginMetadata,
    pub function_names: HashMap<String, FunctionId>,
    pub internal_function_id: HashMap<FunctionId, FunctionId>,
    pub write: interprocess::unnamed_pipe::UnnamedPipeWriter,
}

impl PluginState {
    fn new(
        id: PluginId,
        metadata: PluginMetadata,
        write: interprocess::unnamed_pipe::UnnamedPipeWriter,
    ) -> Self {
        Self {
            id,
            metadata,
            function_names: HashMap::new(),
            internal_function_id: HashMap::new(),
            write,
        }
    }
    #[tracing::instrument(skip(self), fields(plugin.id = %self.id))]
    fn send_function_called(&mut self, msg: FunctionCalled) -> Result<(), std::io::Error> {
        tracing::trace!(?msg, "Sending function call to plugin");
        serde_json::to_writer(&mut self.write, &StewRpcMessage::FunctionCalled(msg))?;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(%invocation_id, plugin.id = %self.id))]
    fn send_response(
        &mut self,
        invocation_id: InvocationId,
        msg: InvocationResponseData,
    ) -> Result<(), std::io::Error> {
        tracing::trace!(%invocation_id, ?msg, "Sending response to plugin");
        serde_json::to_writer(
            &mut self.write,
            &StewRpcMessage::InvocationResponse(InvocationResponse {
                invocation_id,
                kind: msg,
            }),
        )?;
        Ok(())
    }
}
