use futures::Future;
use jsonrpc;
use uuid::Uuid;

use actors::db::namespace;
use rpc;

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Request {
    pub id: Uuid,
}

pub type Response = rpc::namespace::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let msg = namespace::find::Find::from(req);
    meta.db
        .unwrap()
        .send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("namespace find res: {:?}", res);
            match res {
                Ok(res) => Ok(Response::from(res)),
                Err(e) => Err(e.into()),
            }
        })
}
