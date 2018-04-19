use actix::{Addr, Syn};
use jsonrpc::{MetaIoHandler, Metadata};

use actors::DbExecutor;
use rpc::ping::Rpc as PingRpc;

pub mod error;
mod ping;

// TODO: remove Default on new jsonrpc_core version
#[derive(Clone, Default)]
pub struct Meta {
    pub db: Option<Addr<Syn, DbExecutor>>,
}

impl Metadata for Meta {}

pub type Server = MetaIoHandler<Meta>;

pub fn build_server() -> Server {
    let mut io = MetaIoHandler::default();

    let rpc = ping::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    io
}
