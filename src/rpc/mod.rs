use actix::{Addr, Syn};
use jsonrpc::{MetaIoHandler, Metadata};

use actors::DbExecutor;
use rpc::abac_object::Rpc as AbacObjectRpc;
use rpc::auth::Rpc as AuthRpc;
use rpc::ping::Rpc as PingRpc;

pub mod abac_object;
pub mod auth;
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

    let rpc = abac_object::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = auth::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    io
}
