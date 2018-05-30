use uuid::Uuid;

use actors::db;
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
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

impl From<db::abac_object_attr::Create> for NewAbacObjectAttr {
    fn from(msg: db::abac_object_attr::Create) -> Self {
        NewAbacObjectAttr {
            namespace_id: msg.namespace_id,
            object_id: msg.object_id,
            key: msg.key,
            value: msg.value,
        }
    }
}
