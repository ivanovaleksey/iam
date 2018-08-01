use abac::{models::AbacAction, AbacAttribute};
use futures::{future, Future};

use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

#[derive(Debug, Serialize)]
pub struct Response {
    inbound: AbacAttribute,
    outbound: AbacAttribute,
}

impl From<AbacAction> for Response {
    fn from(action: AbacAction) -> Self {
        Response {
            inbound: action.inbound,
            outbound: action.outbound,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::abac_action_attr;
    use rpc::authorize_collection;

    let collection = CollectionKind::AbacAction;
    let operation = OperationKind::Create;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            let inbound_ns_id = req.inbound.namespace_id;
            move |subject_id| {
                authorize_collection(&db, inbound_ns_id, subject_id, collection, operation)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_action_attr::insert::Insert::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac action insert res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
