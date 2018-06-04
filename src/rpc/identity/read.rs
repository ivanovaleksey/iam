use futures::Future;
use jsonrpc;
use uuid::Uuid;

use actors::db;
use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
}

pub type Response = rpc::identity::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let db = meta.db.unwrap();
    let msg = db::identity::find::Find {
        provider: req.provider,
        label: req.label,
        uid: req.uid,
    };

    db.send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("identity find res: {:?}", res);

            let iden = res.map_err(rpc::error::Error::Db)?;
            Ok(Response::from(iden))
        })
}
