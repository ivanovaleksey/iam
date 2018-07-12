use abac::types::AbacAttribute;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{abac_policy, authz::Authz};
use rpc;
use settings;

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub filter: Filter,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Filter {
    pub namespace_ids: Vec<Uuid>,
}

pub type Response = rpc::ListResponse<rpc::abac_policy::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_ids = req.filter.namespace_ids.clone();

            move |subject_id| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

                let futures = namespace_ids.into_iter().map(move |namespace_id| {
                    let msg = Authz {
                        namespace_ids: vec![iam_namespace_id],
                        subject: vec![AbacAttribute::new(
                            iam_namespace_id,
                            UriKind::Account(subject_id),
                        )],
                        object: vec![AbacAttribute::new(namespace_id, CollectionKind::AbacPolicy)],
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::List)],
                    };

                    db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(rpc::ensure_authorized)
                });

                future::join_all(futures)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_policy::select::Select::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("abac policy select res: {:?}", res);
                        Ok(Response::from(res?))
                    })
            }
        })
}
