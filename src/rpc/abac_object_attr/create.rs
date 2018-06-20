use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{abac_object_attr, authz::Authz};
use models::AbacObjectAttr;
use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    namespace_id: Uuid,
    object_id: String,
    key: String,
    value: String,
}

impl From<AbacObjectAttr> for Response {
    fn from(object: AbacObjectAttr) -> Self {
        Response {
            namespace_id: object.namespace_id,
            object_id: object.object_id,
            key: object.key,
            value: object.value,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.namespace_id;
            move |subject_id| {
                //                let msg = Authz {
                //                    namespace_ids: vec![namespace_id],
                //                    subject: subject_id,
                //                    object: format!("namespace.{}", namespace_id),
                //                    action: "execute".to_owned(),
                //                };
                //
                //                db.send(msg)
                //                    .map_err(|_| jsonrpc::Error::internal_error())
                //                    .and_then(rpc::ensure_authorized)
                Ok(())
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_object_attr::insert::Insert::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("abac object insert res: {:?}", res);

                        Ok(Response::from(res?))
                    })
            }
        })
}
