use futures::Future;
use jsonrpc::{self, BoxFuture};
use uuid::Uuid;

use actors::db::authz::Authz;
use rpc;

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
    pub subject: Uuid,
    pub object: String,
    pub action: String,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Response(bool);

impl Response {
    pub fn new(value: bool) -> Self {
        Response(value)
    }
}

#[derive(Clone, Copy)]
pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn authz(&self, meta: rpc::Meta, req: Request) -> BoxFuture<Response> {
        let msg = Authz::from(req);
        let fut = meta
            .db
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
