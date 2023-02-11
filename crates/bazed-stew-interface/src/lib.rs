//! The type definitions used by plugins.
//!
//! It is recommended to use [cbindgen] to generate the necessary headers, when writing a plugin in
//! a different language.
//!
//! A plugin needs to expose the following functions:
//!
//! /// Gets called before the plugin gets loaded.
//! /// Plugin might get discarded, depending on the result.
//! PluginMetadata const *plugin_metadata(void);

//!
//! [cbindgen]: https://github.com/eqrion/cbindgen/

#![warn(unreachable_pub)]

use std::ffi::{c_char, c_void};

mod elk_of_implementation;
mod foo;
pub mod stew_rpc;
pub mod rpc_proto;

/// Metadata about a plugin.
#[repr(C)]
#[derive(Debug)]
pub struct PluginMetadata {
    /// The plugin API version expected by the plugin.
    /// A major version bump indicates some non-backwards compatible change.
    pub api_major: u32,
    /// The plugin API version expected by the plugin.
    /// A minor version bump is backwards compatible.
    pub api_minor: u32,
    /// The name of the plugin.
    /// MUST be a NUL terminated UTF-8 string.
    pub name: *const c_char,
    /// The version of this plugin.
    /// MUST be [semver] compliant, or the plugin will fail to load.
    /// MUST be a NUL terminated valid UTF-8 string.
    ///
    /// [semver]: https://semver.org/
    pub version: *const c_char,
    /// Initialization function called.
    /// Can fail by returning `false`.
    /// In that case the plugin fails to load.
    ///
    /// The plugin is free to store any user data in `data`.
    /// The plugin MUST NOT register any callbacks at this point.
    pub init: extern "C" fn(stew: *const StewVft0, data: *mut *mut c_void) -> bool,
    /// Main function of the plugin.
    /// This is run in its own thread.
    ///
    /// The plugin needs to perform any kind of cleanup within main, before returning.
    pub main: extern "C" fn(stew: *const StewVft0, data: *mut *mut c_void),
}

/// The plugin facing API of stew, version 0.
#[repr(C)]
#[derive(Debug)]
pub struct StewVft0 {
    /// Call a function with a given ID.
    pub call: extern "C" fn(fun: usize, arg: *mut c_void) -> usize,
    /// Register a new function for others to call.
    ///
    /// TODO: stew errors vs plugin errors
    pub register: extern "C" fn(
        name: *const c_char,
        fun: extern "C" fn(data: *mut *mut c_void, arg: *mut c_void) -> usize,
    ) -> usize,
    /// Get a previously registered function (from a different plugin).
    /// `id` is the ID you will use to refer to to this function from now on.
    ///
    /// TODO: limitations on `id`
    /// TODO: error codes
    pub get_fn: extern "C" fn(name: *const c_char, id: usize) -> usize,
    /// Load a plugin from the load path.
    ///
    /// TODO: error codes
    /// TODO: expose load path
    /// TODO: probably offer a second one that ignores load path
    pub load_plugin: extern "C" fn(name: *const c_char, version: *const c_char) -> usize,
}
