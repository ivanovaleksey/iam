use chrono::NaiveDateTime;
use uuid::Uuid;

use models::namespace::Namespace;
use schema::abac_policy;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace)]
#[table_name = "abac_policy"]
pub struct AbacPolicy {
    id: Uuid,
    namespace_id: Uuid,
    subject_value: String,
    object_value: String,
    action_value: String,
    issued_at: NaiveDateTime,
    not_before: Option<NaiveDateTime>,
    expired_at: Option<NaiveDateTime>,
}
