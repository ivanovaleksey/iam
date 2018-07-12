use abac::types::AbacAttribute;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{authz::Authz, namespace};
use rpc;
use settings;

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub filter: Filter,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Filter {
    pub account_id: Uuid,
}

pub type Response = rpc::ListResponse<rpc::namespace::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind, UriKind};

    let account_id = req.filter.account_id;
    let iam_namespace_id = settings::iam_namespace_id();

    let objects = vec![
        AbacAttribute::new(iam_namespace_id, CollectionKind::Namespace),
        AbacAttribute::new(iam_namespace_id, UriKind::Account(account_id)),
    ];

    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let objects = objects.clone();

            move |subject_id| {
                let msg = Authz {
                    namespace_ids: vec![iam_namespace_id],
                    subject: vec![AbacAttribute::new(
                        iam_namespace_id,
                        UriKind::Account(subject_id),
                    )],
                    object: objects,
                    action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::List)],
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();

            move |_| {
                use actors::db::object_list::ObjectList;

                let msg = ObjectList {
                    objects,
                    offset: 0,
                    limit: 100,
                };
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        let attrs = res.map_err(rpc::error::Error::Db)?;
                        let ids = attrs
                            .into_iter()
                            .filter_map(|attr| {
                                let mut kv = attr.value.splitn(2, '/');
                                match (kv.next(), kv.next()) {
                                    (Some("namespace"), Some(v)) => Uuid::parse_str(v).ok(),
                                    _ => None,
                                }
                            })
                            .collect::<Vec<_>>();

                        Ok(ids)
                    })
            }
        })
        .and_then({
            let db = meta.db.unwrap();

            move |ids| {
                let msg = namespace::select::Select { ids };
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("namespace select res: {:?}", res);
                        Ok(Response::from(res?))
                    })
            }
        })
}
