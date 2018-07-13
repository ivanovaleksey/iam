use futures::{future, Future};

use rpc;

pub type Request = rpc::abac_policy::read::Request;
pub type Response = rpc::abac_policy::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::abac_policy;
    use rpc::authorize_collection;

    let collection = CollectionKind::AbacPolicy;
    let operation = OperationKind::Delete;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            let ns_id = req.namespace_id;
            move |subject_id| authorize_collection(&db, ns_id, subject_id, collection, operation)
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_policy::delete::Delete::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac policy delete res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
