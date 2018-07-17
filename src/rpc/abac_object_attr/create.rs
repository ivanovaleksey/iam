use abac::{models::AbacObject, types::AbacAttribute};
use futures::future::{self, Future};
use jsonrpc;

use actors::db::{abac_object_attr, authz::Authz};
use rpc;
use settings;

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

impl From<AbacObject> for Response {
    fn from(object: AbacObject) -> Self {
        Response {
            inbound: object.inbound,
            outbound: object.outbound,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.inbound.namespace_id;
            move |subject_id| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

                let msg = Authz {
                    namespace_ids: vec![iam_namespace_id],
                    subject: vec![AbacAttribute::new(
                        iam_namespace_id,
                        UriKind::Account(subject_id),
                    )],
                    object: vec![
                        AbacAttribute::new(iam_namespace_id, UriKind::Namespace(namespace_id)),
                        AbacAttribute::new(iam_namespace_id, CollectionKind::AbacObject),
                    ],
                    action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Create)],
                };

                db.send(msg).from_err().and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_object_attr::insert::Insert::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac object insert res: {:?}", res);

                    Ok(Response::from(res?))
                })
            }
        })
        .from_err()
}
