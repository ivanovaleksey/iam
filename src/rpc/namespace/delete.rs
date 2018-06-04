use futures::Future;
use jsonrpc;

use actors::db::namespace;
use rpc;

pub type Request = rpc::namespace::read::Request;
pub type Response = rpc::namespace::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let msg = namespace::delete::Delete::from(req);
    meta.db
        .unwrap()
        .send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("namespace delete res: {:?}", res);
            match res {
                Ok(res) => Ok(Response::from(res)),
                Err(e) => Err(e.into()),
            }
        })
}
