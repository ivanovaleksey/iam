use uuid::Uuid;

use actors::db::abac_action;
use models::Namespace;
use schema::abac_action_attr;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace)]
#[primary_key(namespace_id, action_id, key, value)]
#[table_name = "abac_action_attr"]
pub struct AbacActionAttr {
    pub namespace_id: Uuid,
    pub action_id: String,
    pub key: String,
    pub value: String,
}

#[derive(AsChangeset, Insertable, Debug)]
#[table_name = "abac_action_attr"]
pub struct NewAbacActionAttr {
    namespace_id: Uuid,
    action_id: String,
    key: String,
    value: String,
}

impl From<abac_action::Create> for NewAbacActionAttr {
    fn from(msg: abac_action::Create) -> Self {
        NewAbacActionAttr {
            namespace_id: msg.namespace_id,
            action_id: msg.action_id,
            key: msg.key,
            value: msg.value,
        }
    }
}