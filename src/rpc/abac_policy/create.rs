use chrono::NaiveDateTime;
use futures::Future;
use jsonrpc;
use uuid::Uuid;

use actors::db::abac_policy;
use models::AbacPolicy;
use rpc;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_id: Uuid,
    pub subject_namespace_id: Uuid,
    pub subject_key: String,
    pub subject_value: String,
    pub object_namespace_id: Uuid,
    pub object_key: String,
    pub object_value: String,
    pub action_namespace_id: Uuid,
    pub action_key: String,
    pub action_value: String,
    pub not_before: Option<NaiveDateTime>,
    pub expired_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    namespace_id: Uuid,
    subject_namespace_id: Uuid,
    subject_key: String,
    subject_value: String,
    object_namespace_id: Uuid,
    object_key: String,
    object_value: String,
    action_namespace_id: Uuid,
    action_key: String,
    action_value: String,
    created_at: NaiveDateTime,
    not_before: Option<NaiveDateTime>,
    expired_at: Option<NaiveDateTime>,
}

impl From<AbacPolicy> for Response {
    fn from(policy: AbacPolicy) -> Self {
        Response {
            namespace_id: policy.namespace_id,
            subject_namespace_id: policy.subject_namespace_id,
            subject_key: policy.subject_key,
            subject_value: policy.subject_value,
            object_namespace_id: policy.object_namespace_id,
            object_key: policy.object_key,
            object_value: policy.object_value,
            action_namespace_id: policy.action_namespace_id,
            action_key: policy.action_key,
            action_value: policy.action_value,
            created_at: policy.created_at,
            not_before: policy.not_before,
            expired_at: policy.expired_at,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let msg = abac_policy::insert::Insert::from(req);
    meta.db
        .unwrap()
        .send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("abac policy insert res: {:?}", res);
            Ok(Response::from(res?))
        })
}
