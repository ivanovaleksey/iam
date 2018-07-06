use abac::types::AbacAttribute;
use chrono::NaiveDateTime;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{authz::Authz, namespace};
use models::{Namespace, NewNamespace};
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct Response {
    id: Uuid,
    label: String,
    account_id: Uuid,
    enabled: bool,
    created_at: NaiveDateTime,
}

impl From<Namespace> for Response {
    fn from(namespace: Namespace) -> Self {
        Response {
            id: namespace.id,
            label: namespace.label,
            account_id: namespace.account_id,
            enabled: namespace.enabled,
            created_at: namespace.created_at,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let iam_namespace_id = settings::iam_namespace_id();

                let msg = Authz {
                    namespace_ids: vec![iam_namespace_id],
                    subject: vec![AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", subject_id),
                    }],
                    object: vec![AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "type".to_owned(),
                        value: "namespace".to_owned(),
                    }],
                    action: vec![AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "operation".to_owned(),
                        value: "create".to_owned(),
                    }],
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let changeset = NewNamespace::from(req);
                let msg = namespace::insert::Insert(changeset);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("namespace insert res: {:?}", res);
                        let namespace = res.map_err(rpc::error::Error::Db)?;
                        Ok(Response::from(namespace))
                    })
            }
        })
}
