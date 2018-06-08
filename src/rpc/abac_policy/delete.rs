use futures::Future;
use jsonrpc;

use actors::db::abac_policy;
use rpc;

pub type Request = rpc::abac_policy::read::Request;
pub type Response = rpc::abac_policy::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let msg = abac_policy::delete::Delete::from(req);
    meta.db
        .unwrap()
        .send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("abac policy delete res: {:?}", res);
            Ok(Response::from(res?))
        })
}
