use abac::{models::AbacPolicy, types::AbacAttribute};
use futures::{future, Future};
use uuid::Uuid;

use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_id: Uuid,
    pub subject: Vec<AbacAttribute>,
    pub object: Vec<AbacAttribute>,
    pub action: Vec<AbacAttribute>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub namespace_id: Uuid,
    pub subject: Vec<AbacAttribute>,
    pub object: Vec<AbacAttribute>,
    pub action: Vec<AbacAttribute>,
}

impl From<AbacPolicy> for Response {
    fn from(policy: AbacPolicy) -> Self {
        Response {
            namespace_id: policy.namespace_id,
            subject: policy.subject,
            object: policy.object,
            action: policy.action,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::abac_policy;
    use rpc::authorize_collection;

    let collection = CollectionKind::AbacPolicy;
    let operation = OperationKind::Create;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            let ns_id = req.namespace_id;
            move |subject_id| authorize_collection(&db, ns_id, subject_id, collection, operation)
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_policy::insert::Insert::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac policy insert res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
