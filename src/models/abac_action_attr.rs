use uuid::Uuid;

use models::namespace::Namespace;
use schema::abac_action_attr;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace)]
#[primary_key(namespace_id, value, action_id)]
#[table_name = "abac_action_attr"]
pub struct AbacActionAttr {
    namespace_id: Uuid,
    action_id: String,
    value: String,
}
