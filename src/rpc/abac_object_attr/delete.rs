use futures::{future, Future};

use rpc;

pub type Request = rpc::abac_object_attr::create::Request;
pub type Response = rpc::abac_object_attr::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::abac_object_attr;
    use rpc::authorize_collection;

    let Request { inbound, outbound } = req.clone();

    let inbound_ns_id = inbound.namespace_id;
    let outbound_ns_id = outbound.namespace_id;

    let collection = CollectionKind::AbacObject;
    let operation = OperationKind::Delete;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                authorize_collection(&db, inbound_ns_id, subject_id, collection, operation).or_else(
                    move |_| {
                        authorize_collection(&db, outbound_ns_id, subject_id, collection, operation)
                    },
                )
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_object_attr::delete::Delete::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac object delete res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
