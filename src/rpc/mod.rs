use actix::{Addr, Syn};
use jsonrpc::{MetaIoHandler, Metadata};
use serde::de::{self, Deserialize, Deserializer};

use std::{fmt, str};

use actors::DbExecutor;
use rpc::abac_object::Rpc as AbacObjectRpc;
use rpc::abac_subject::Rpc as AbacSubjectRpc;
use rpc::auth::Rpc as AuthRpc;
use rpc::ping::Rpc as PingRpc;

pub mod abac_object;
pub mod abac_subject;
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

    let rpc = auth::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = abac_subject::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = abac_object::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    io
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ListRequest<F>
where
    F: str::FromStr,
    F::Err: fmt::Display,
{
    #[serde(rename = "fq")]
    pub filter: ListRequestFilter<F>,
}

impl<F> ListRequest<F>
where
    F: str::FromStr,
    F::Err: fmt::Display,
{
    pub fn new(filter: F) -> Self {
        ListRequest {
            filter: ListRequestFilter(filter),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ListRequestFilter<F>(pub F);

impl<'de, F> Deserialize<'de> for ListRequestFilter<F>
where
    F: str::FromStr,
    F::Err: fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let filter = s.parse().map_err(de::Error::custom)?;
        let filter = ListRequestFilter(filter);
        Ok(filter)
    }
}
