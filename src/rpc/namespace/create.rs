use abac::AbacAttribute;
use chrono::{DateTime, Utc};
use futures::future::{self, Future};
use uuid::Uuid;

use actors::db::{authz::Authz, namespace};
use models::{Namespace, NewNamespace};
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub data: RequestData,
}

#[derive(Debug, Deserialize)]
pub struct RequestData {
    pub label: String,
    pub account_id: Uuid,
}

pub type Response = rpc::Response<Uuid, ResponseData>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub label: String,
    pub account_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<Request> for NewNamespace {
    fn from(msg: Request) -> Self {
        let data = msg.data;
        NewNamespace {
            label: data.label,
            account_id: data.account_id,
        }
    }
}

impl From<Namespace> for Response {
    fn from(namespace: Namespace) -> Self {
        Response {
            id: namespace.id,
            data: ResponseData {
                label: namespace.label,
                account_id: namespace.account_id,
                created_at: namespace.created_at,
            },
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

                let msg = Authz {
                    namespace_ids: vec![iam_namespace_id],
                    subject: vec![AbacAttribute::new(
                        iam_namespace_id,
                        UriKind::Account(subject_id),
                    )],
                    object: vec![AbacAttribute::new(
                        iam_namespace_id,
                        CollectionKind::Namespace,
                    )],
                    action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Create)],
                };

                db.send(msg).from_err().and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let changeset = NewNamespace::from(req);
                let msg = namespace::insert::Insert(changeset);
                db.send(msg).from_err().and_then(|res| {
                    debug!("namespace insert res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
