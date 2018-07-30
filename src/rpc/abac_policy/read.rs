use futures::{future, Future};

use rpc;

pub type Request = rpc::abac_policy::create::Request;
pub type Response = rpc::abac_policy::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::abac_policy;
    use rpc::authorize_collection;

    let collection = CollectionKind::AbacPolicy;
    let operation = OperationKind::Read;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            let ns_id = req.namespace_id;
            move |subject_id| authorize_collection(&db, ns_id, subject_id, collection, operation)
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_policy::find::Find::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac policy find res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
