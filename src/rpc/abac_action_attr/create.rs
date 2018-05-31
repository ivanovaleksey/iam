use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{abac_action_attr, authz::Authz};
use models::AbacActionAttr;
use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_id: Uuid,
    pub action_id: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    namespace_id: Uuid,
    action_id: String,
    key: String,
    value: String,
}

impl From<AbacActionAttr> for Response {
    fn from(attr: AbacActionAttr) -> Self {
        Response {
            namespace_id: attr.namespace_id,
            action_id: attr.action_id,
            key: attr.key,
            value: attr.value,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = meta.subject.ok_or(rpc::error::Error::Forbidden.into());
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.namespace_id;
            move |subject_id| {
                let msg = Authz {
                    namespace_ids: vec![namespace_id],
                    subject: subject_id,
                    object: format!("namespace.{}", namespace_id),
                    action: "execute".to_owned(),
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        if res? {
                            Ok(())
                        } else {
                            Err(rpc::error::Error::Forbidden)?
                        }
                    })
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_action_attr::insert::Insert::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("abac action insert res: {:?}", res);

                        Ok(Response::from(res?))
                    })
            }
        })
}
