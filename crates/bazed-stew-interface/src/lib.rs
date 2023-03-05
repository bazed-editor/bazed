#![warn(unreachable_pub)]

pub mod ipc_connection;
pub mod rpc_proto;
pub mod stew_rpc;

use std::os::fd::FromRawFd;

use interprocess::unnamed_pipe::{UnnamedPipeReader, UnnamedPipeWriter};
use ipc_connection::{UnnamedPipeJsonReader, UnnamedPipeJsonWriter};
use rpc_proto::StewRpcCall;
pub use semver;
use stew_rpc::StewClient;

pub type IpcStewClient<D> = StewClient<ipc_connection::UnnamedPipeJsonWriter<StewRpcCall>, D>;

pub fn init_client<D: Send + Sync + 'static>(state: D) -> IpcStewClient<D> {
    let writer_fd = std::env::args().nth(1).unwrap().parse().unwrap();
    let writer = unsafe { UnnamedPipeWriter::from_raw_fd(writer_fd) };
    let writer = UnnamedPipeJsonWriter::new(writer);

    let reader_fd = std::env::args().nth(2).unwrap().parse().unwrap();
    let reader = unsafe { UnnamedPipeReader::from_raw_fd(reader_fd) };
    let reader = UnnamedPipeJsonReader::new(reader);

    //let _plugin_id: PluginId = PluginId(std::env::args().nth(3).unwrap().parse().unwrap());

    StewClient::start(writer, reader, state)
}
