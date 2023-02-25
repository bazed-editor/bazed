//! # Stew RPC protocol
//!
//! This module contains the message types used to communicate between the plugin system and plugins.
//!
//! ## Plugin initialization procedure
//!
//! ```mermaid
//! sequenceDiagram
//!     Cucumber ->>+ Stew : load_plugin(name: "banana", v: ">=2.6.9")
//!     Stew ->> Banana : start process
//!     Banana ->> Stew : Metadata { ... }
//!     Banana ->> Stew : register_fn(name: "applepie", internal_id: 69)
//!     Banana ->> Stew : PluginReady
//!     Stew ->>- Cucumber : PluginLoaded(plugin_id: 5, version: 2.7.0)
//!     Cucumber ->>+ Stew : get_fn(plugin_id: 5, fn: "applepie")
//!     Stew ->>- Cucumber : fn_id: 123
//!     
//!     Cucumber ->>+ Stew : call(fn_id: 123, invocation_id: 1)
//!     Stew ->>+ Banana : call(internal_id: 69, caller_id: 55, invocation_id: 1)
//!     Banana ->>- Stew : function_returned(caller_id: 55, invocation_id: 1, data: "lmao")
//!     Stew ->>+ Cucumber : call_result(invocation_id: 1, data: "lmao")
//! ```
//!
//! TODO: Deal with invocation timeouts
//! TODO: Figure out how to include tracing information here so we can get distributed tracing, somehow

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PluginId(pub Uuid);

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub Uuid);

impl FunctionId {
    pub fn gen() -> Self {
        Self(Uuid::new_v4())
    }
}

#[repr(transparent)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvocationId(pub Uuid);

impl InvocationId {
    pub fn gen() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Metadata about a plugin.
#[repr(C)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginMetadata {
    /// The plugin API version expected by the plugin.
    /// A major version bump indicates some non-backwards compatible change.
    pub api_major: u32,
    /// The plugin API version expected by the plugin.
    /// A minor version bump is backwards compatible.
    pub api_minor: u32,
    /// The name of the plugin.
    pub name: String,
    /// The version of this plugin.
    /// MUST be [semver] compliant, or the plugin will fail to load.
    ///
    /// [semver]: https://semver.org/
    pub version: String,
}

/// Calls from the plugin to the plugin system
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum StewRpcCall {
    /// Register a new function for others to call.
    RegisterFunction {
        /// The name of this function.
        /// Must be a valid unicode identifier as per [UAX31-D1](https://unicode.org/reports/tr31/#D1)
        fn_name: String,
        /// The ID internally associated with this function.
        /// We use IDs here instead of just the name to reduce the amount
        /// of data we have to send over the wire.
        internal_id: FunctionId,
    },
    /// Get a previously registered function (from a different plugin).
    GetFunction {
        /// The ID of the plugin instance with the function.
        plugin_id: PluginId,
        /// The name of the function to get.
        fn_name: String,
        invocation_id: InvocationId,
    },
    /// Call a function with a given ID.
    CallFunction {
        /// The function ID, previously retrieved via [StewRpcCall::GetFunction].
        fn_id: FunctionId,
        args: serde_json::Value,
        /// The ID of the invocation. used to match the return value to the call.
        /// When set, indicates that a response is to be expected. When not set,
        /// no response should be expected.
        invocation_id: Option<InvocationId>,
    },
    /// Should be sent when a function from this plugin that was called via
    /// [StewRpcMessage::FunctionCalled] returns, and an [InvocationId] was provided.
    FunctionReturn {
        /// The id of the plugin that called the function.
        /// Provided by the [StewRpcMessage::FunctionCalled] message.
        caller_id: PluginId,
        /// The return value of the function.
        return_value: FunctionResult,
        /// The ID of the invocation, used to match the return value to the call.
        invocation_id: InvocationId,
    },

    /// Load a plugin from the load path.
    /// Should result in a [StewRpcMessage::PluginLoaded] message.
    LoadPlugin {
        /// Name of the plugin to load
        name: String,
        /// Version specification, see [semver](https://docs.rs/semver/1.0.16/semver/struct.VersionReq.html) for details.
        version_requirement: String,
        invocation_id: InvocationId,
    },

    /// Sent when the plugin started, contains metadata about the plugin.
    /// This must be sent before any other calls.
    Metadata(PluginMetadata),

    /// Should be sent when this plugin is done initializing.
    /// From this point onwards, the registered functions will be made available to other plugins.
    PluginReady,
}

/// Messages from the plugin system to the plugin
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum StewRpcMessage {
    /// A function call from another plugin.
    FunctionCalled(FunctionCalled),
    InvocationResponse(InvocationResponse),
}

/// A function call from another plugin.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct FunctionCalled {
    /// The internal ID of the function that was called.
    pub internal_id: FunctionId,
    pub args: serde_json::Value,
    /// The ID of the plugin that called the function.
    /// Must be included in the return value response.
    pub caller_id: PluginId,
    /// The ID of the invocation.
    /// When set, this must be included in the return value response
    /// such that the caller can match the response to the invocation.
    ///
    /// Any function call should yield a [StewRpcCall::FunctionReturn] message
    pub invocation_id: Option<InvocationId>,
}

/// A response to some invocation (any call that expects a result via some [InvocationId])
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct InvocationResponse {
    pub invocation_id: InvocationId,
    pub kind: InvocationResponseData,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum InvocationResponseData {
    /// A function called by this plugin returned a value.
    FunctionReturned(FunctionResult),
    /// Result of [StewRpcCall::GetFunction]
    GotFunctionId(FunctionId),
    /// Result of [StewRpcCall::LoadPlugin], sent when the plugin is loaded.
    PluginLoaded {
        /// The ID of the plugin that was loaded.
        plugin_id: PluginId,
        /// The exact version of the plugin that was loaded.
        version: String,
    },
    /// Some invocation of stew failed.
    InvocationFailed(serde_json::Value),
}

/// The result of a function call, either a value or an error.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FunctionResult {
    /// The function returned a value.
    Value(serde_json::Value),
    /// The function returned an error.
    Error(serde_json::Value),
}

impl<T, E> From<Result<T, E>> for FunctionResult
where
    T: Serialize,
    E: Serialize,
{
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(v) => FunctionResult::Value(serde_json::to_value(v).unwrap()),
            Err(e) => FunctionResult::Error(serde_json::to_value(e).unwrap()),
        }
    }
}

impl FunctionResult {
    pub fn parse_into_result<T: DeserializeOwned, E: DeserializeOwned>(
        self,
    ) -> Result<Result<T, E>, serde_json::Error> {
        match self {
            FunctionResult::Value(v) => Ok(Ok(serde_json::from_value(v)?)),
            FunctionResult::Error(e) => Ok(Err(serde_json::from_value(e)?)),
        }
    }
}
