use futures::{future, Future};
use uuid::Uuid;

use rpc;

#[derive(Debug, Deserialize)]
pub struct Filter {
    pub namespace_ids: Vec<Uuid>,
}

pub type Request = rpc::ListRequest<Filter>;
pub type Response = rpc::ListResponse<rpc::abac_policy::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::abac_policy;
    use rpc::authorize_collection;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_ids = req.filter.namespace_ids.clone();

            move |subject_id| {
                let collection = CollectionKind::AbacPolicy;
                let operation = OperationKind::List;

                let futures = namespace_ids.into_iter().map(move |ns_id| {
                    authorize_collection(&db, ns_id, subject_id, collection, operation)
                });

                future::join_all(futures)
            }
        })
        .and_then({
            let limit = req.pagination.limit;
            move |_| rpc::pagination::check_limit(limit)
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_policy::select::Select {
                    namespace_ids: req.filter.namespace_ids,
                    limit: req.pagination.limit,
                    offset: req.pagination.offset,
                };
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac policy select res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
