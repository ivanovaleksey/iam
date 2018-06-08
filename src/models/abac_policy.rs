use chrono::NaiveDateTime;
use uuid::Uuid;

use actors::db;
use models::Namespace;
use schema::abac_policy;

#[derive(Associations, Identifiable, Queryable, Debug, Deserialize)]
#[belongs_to(Namespace)]
#[primary_key(
    namespace_id, subject_namespace_id, subject_key, subject_value, object_namespace_id, object_key,
    object_value, action_namespace_id, action_key, action_value
)]
#[table_name = "abac_policy"]
pub struct AbacPolicy {
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
    pub created_at: NaiveDateTime,
    pub not_before: Option<NaiveDateTime>,
    pub expired_at: Option<NaiveDateTime>,
}

#[derive(AsChangeset, Insertable, Debug)]
#[table_name = "abac_policy"]
pub struct NewAbacPolicy {
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

impl From<db::abac_policy::insert::Insert> for NewAbacPolicy {
    fn from(msg: db::abac_policy::insert::Insert) -> Self {
        NewAbacPolicy {
            namespace_id: msg.namespace_id,
            subject_namespace_id: msg.subject_namespace_id,
            subject_key: msg.subject_key,
            subject_value: msg.subject_value,
            object_namespace_id: msg.object_namespace_id,
            object_key: msg.object_key,
            object_value: msg.object_value,
            action_namespace_id: msg.action_namespace_id,
            action_key: msg.action_key,
            action_value: msg.action_value,
            not_before: msg.not_before,
            expired_at: msg.expired_at,
        }
    }
}
