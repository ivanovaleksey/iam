use chrono::NaiveDateTime;
use uuid::Uuid;

use models::Namespace;
use schema::abac_policy;

#[derive(Associations, Identifiable, Queryable, Debug)]
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
    pub issued_at: NaiveDateTime,
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
