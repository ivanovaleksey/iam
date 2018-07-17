use abac::types::AbacAttribute;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{authz::Authz, identity};
use models::identity::PrimaryKey;
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub filter: Filter,
}

#[derive(Debug, Deserialize)]
pub struct Filter {
    pub provider: Option<Uuid>,
    pub account_id: Option<Uuid>,
}

pub type Response = rpc::ListResponse<rpc::identity::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind, UriKind};

    let iam_namespace_id = settings::iam_namespace_id();

    let mut objects = vec![AbacAttribute::new(
        iam_namespace_id,
        CollectionKind::Identity,
    )];

    if let Some(provider) = req.filter.provider {
        objects.push(AbacAttribute::new(
            iam_namespace_id,
            UriKind::Namespace(provider),
        ));
    }

    if let Some(account_id) = req.filter.account_id {
        objects.push(AbacAttribute::new(
            iam_namespace_id,
            UriKind::Account(account_id),
        ));
    }

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
            let db = meta.db.clone().unwrap();

            move |_| {
                use actors::db::object_list::ObjectList;

                let msg = ObjectList {
                    objects,
                    offset: 0,
                    limit: 100,
                };
                db.send(msg).from_err().and_then(|res| {
                    let ids = res?.into_iter()
                        .filter_map(|attr| {
                            let mut kv = attr.value.splitn(2, '/');
                            match (kv.next(), kv.next()) {
                                (Some("identity"), Some(v)) => v.parse::<PrimaryKey>().ok(),
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
                let msg = identity::select::Select::ByIds(ids);
                db.send(msg).from_err().and_then(|res| {
                    debug!("identity select res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
        .from_err()
}
