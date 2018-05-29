use chrono::NaiveDateTime;
use futures::Future;
use jsonrpc;
use uuid::Uuid;

use actors::db::namespace;
use models::Namespace;
use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct Response {
    id: Uuid,
    label: String,
    account_id: Uuid,
    enabled: bool,
    created_at: NaiveDateTime,
}

impl From<Namespace> for Response {
    fn from(namespace: Namespace) -> Self {
        Response {
            id: namespace.id,
            label: namespace.label,
            account_id: namespace.account_id,
            enabled: namespace.enabled,
            created_at: namespace.created_at,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let msg = namespace::insert::Insert::from(req);
    meta.db
        .unwrap()
        .send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("namespace insert res: {:?}", res);
            match res {
                Ok(res) => Ok(Response::from(res)),
                Err(e) => Err(e.into()),
            }
        })
}
