use futures::{future, Future};

use rpc;

pub type Request = rpc::ListRequest;
pub type Response = rpc::ListResponse<rpc::abac_action_attr::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::abac_action_attr;
    use rpc::authorize_collection;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_ids = req.filter.namespace_ids.clone();

            move |subject_id| {
                let collection = CollectionKind::AbacAction;
                let operation = OperationKind::List;

                let futures = namespace_ids.into_iter().map(move |ns_id| {
                    authorize_collection(&db, ns_id, subject_id, collection, operation)
                });

                future::join_all(futures)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_action_attr::select::Select {
                    namespace_ids: req.filter.namespace_ids,
                    key: req.filter.key,
                };
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac action select res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
