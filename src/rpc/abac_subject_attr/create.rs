use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{abac_subject_attr, authz::Authz};
use models::AbacSubjectAttr;
use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_id: Uuid,
    pub subject_id: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    namespace_id: Uuid,
    subject_id: Uuid,
    key: String,
    value: String,
}

impl From<AbacSubjectAttr> for Response {
    fn from(attr: AbacSubjectAttr) -> Self {
        Response {
            namespace_id: attr.namespace_id,
            subject_id: attr.subject_id,
            key: attr.key,
            value: attr.value,
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
                let msg = abac_subject_attr::insert::Insert::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("abac subject insert res: {:?}", res);

                        Ok(Response::from(res?))
                    })
            }
        })
}
