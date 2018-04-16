use uuid::Uuid;

use models::namespace::Namespace;
use schema::abac_object_attr;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace)]
#[primary_key(namespace_id, value, object_id)]
#[table_name = "abac_object_attr"]
pub struct AbacObjectAttr {
    namespace_id: Uuid,
    object_id: String,
    value: String,
}
