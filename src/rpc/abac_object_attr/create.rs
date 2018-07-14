use abac::{models::AbacObject, types::AbacAttribute};
use futures::{future, Future};

use rpc;

#[derive(Debug, Deserialize, Clone)]
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

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind, UriKind};
    use actors::db::{abac_object_attr, abac_object_target};
    use rpc::authorize_collection;
    use settings;

    let Request { inbound, outbound } = req.clone();

    let inbound_ns_id = inbound.namespace_id;
    let outbound_ns_id = outbound.namespace_id;

    let collection = CollectionKind::AbacObject;
    let operation = OperationKind::Create;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                authorize_collection(&db, inbound_ns_id, subject_id, collection, operation).or_else(
                    move |_| {
                        authorize_collection(&db, outbound_ns_id, subject_id, collection, operation)
                            .and_then(move |_| {
                                let iam_namespace_id = settings::iam_namespace_id();
                                let outbound_ns_uri = AbacAttribute::new(
                                    iam_namespace_id,
                                    UriKind::Namespace(outbound_ns_id),
                                );
                                let msg =
                                    abac_object_target::HasTarget(vec![inbound], outbound_ns_uri);
                                db.send(msg).from_err().and_then(rpc::ensure_authorized)
                            })
                    },
                )
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
}
