use futures::Future;
use jsonrpc;
use uuid::Uuid;

use actors::db;
use models::Account;
use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub id: Uuid,
}

impl From<Account> for Response {
    fn from(account: Account) -> Self {
        Response { id: account.id }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let db = meta.db.unwrap();
    let msg = db::account::find::Find::from(req);

    db.send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("account find res: {:?}", res);

            let account = res.map_err(rpc::error::Error::Db)?;
            Ok(Response::from(account))
        })
}
