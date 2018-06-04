use futures::Future;
use jsonrpc;
use uuid::Uuid;

use actors::db::namespace;
use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Uuid,
    pub label: Option<String>,
    pub enabled: Option<bool>,
}

pub type Response = rpc::namespace::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let msg = namespace::update::Update::from(req);
    meta.db
        .unwrap()
        .send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("namespace update res: {:?}", res);
            match res {
                Ok(res) => Ok(Response::from(res)),
                Err(e) => Err(e.into()),
            }
        })
}
