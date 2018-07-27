use abac::AbacAttribute;
use futures::future::{self, Future};
use uuid::Uuid;

use actors::db::{authz::Authz, namespace};
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Filter {
    pub account_id: Uuid,
}

pub type Request = rpc::ListRequest<Filter>;
pub type Response = rpc::ListResponse<rpc::namespace::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
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

                db.send(msg).from_err().and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let limit = req.pagination.limit;
            move |_| rpc::pagination::check_limit(limit)
        })
        .and_then({
            let db = meta.db.clone().unwrap();

            move |_| {
                use actors::db::object_list::ObjectList;

                let msg = ObjectList {
                    objects,
                    limit: req.pagination.limit,
                    offset: req.pagination.offset,
                };
                db.send(msg).from_err().and_then(|res| {
                    let ids = res?
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
                db.send(msg).from_err().and_then(|res| {
                    debug!("namespace select res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
