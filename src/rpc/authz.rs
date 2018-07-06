use abac::types::AbacAttribute;
use futures::Future;
use jsonrpc::{self, BoxFuture};
use uuid::Uuid;

use actors::db::authz::Authz;
use rpc;
use settings;

build_rpc_trait! {
    pub trait Rpc {
        type Metadata;

        #[rpc(meta, name = "authorize")]
        fn authz(&self, Self::Metadata, Request) -> BoxFuture<Response>;
    }
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_ids: Vec<Uuid>,
    pub subject: Vec<AbacAttribute>,
    pub object: Vec<AbacAttribute>,
    pub action: Vec<AbacAttribute>,
}

#[derive(Debug, Serialize)]
pub struct Response(bool);

impl Response {
    pub fn new(value: bool) -> Self {
        Response(value)
    }
}

#[allow(missing_debug_implementations)]
pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn authz(&self, meta: rpc::Meta, req: Request) -> BoxFuture<Response> {
        let iam_namespace_id = settings::iam_namespace_id();

        let mut msg = Authz::from(req);
        msg.namespace_ids.push(iam_namespace_id);
        msg.namespace_ids.dedup();

        let fut = meta.db
            .unwrap()
            .send(msg)
            .map_err(|_| jsonrpc::Error::internal_error())
            .and_then(|res| {
                let res = res.map_err(rpc::error::Error::Db)?;
                Ok(Response::new(res))
            });

        Box::new(fut)
    }
}
