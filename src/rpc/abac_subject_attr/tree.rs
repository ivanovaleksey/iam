use futures::{future, Future};

use rpc;

pub type Request = rpc::TreeRequest;
pub type Response = rpc::TreeResponse;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::tree;
    use rpc::authorize_collection;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            let ns_id = req.filter.attribute.namespace_id;

            move |subject_id| {
                let collection = CollectionKind::AbacSubject;
                let operation = OperationKind::List;

                authorize_collection(&db, ns_id, subject_id, collection, operation)
            }
        })
        .and_then({
            let limit = req.pagination.limit;
            move |_| rpc::pagination::check_limit(limit)
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = tree::Tree(
                    tree::CollectionKind::AbacSubject,
                    tree::Select {
                        direction: req.filter.direction,
                        attribute: req.filter.attribute,
                        limit: req.pagination.limit,
                        offset: req.pagination.offset,
                    },
                );
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac subject select res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
