use futures::{future, Future};
use uuid::Uuid;

use rpc;

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub filter: Filter,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Filter {
    pub namespace_ids: Vec<Uuid>,
}

pub type Response = rpc::ListResponse<rpc::abac_subject_attr::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db::abac_subject_attr;
    use rpc::authorize_collection;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_ids = req.filter.namespace_ids.clone();

            move |subject_id| {
                let collection = CollectionKind::AbacSubject;
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
                let msg = abac_subject_attr::select::Select::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac subject select res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
