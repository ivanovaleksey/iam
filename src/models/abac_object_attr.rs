use uuid::Uuid;

use actors::db::abac_object;
use models::Namespace;
use schema::abac_object_attr;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace)]
#[primary_key(namespace_id, object_id, key, value)]
#[table_name = "abac_object_attr"]
pub struct AbacObjectAttr {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

#[derive(AsChangeset, Insertable, Debug)]
#[table_name = "abac_object_attr"]
pub struct NewAbacObjectAttr {
    namespace_id: Uuid,
    object_id: String,
    key: String,
    value: String,
}

impl From<abac_object::Create> for NewAbacObjectAttr {
    fn from(msg: abac_object::Create) -> Self {
        NewAbacObjectAttr {
            namespace_id: msg.namespace_id,
            object_id: msg.object_id,
            key: msg.key,
            value: msg.value,
        }
    }
}
