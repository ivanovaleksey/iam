use uuid::Uuid;

use actors::db;
use models::Namespace;
use schema::abac_subject_attr;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace)]
#[primary_key(namespace_id, subject_id, key, value)]
#[table_name = "abac_subject_attr"]
pub struct AbacSubjectAttr {
    pub namespace_id: Uuid,
    pub subject_id: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(AsChangeset, Insertable, Debug)]
#[table_name = "abac_subject_attr"]
pub struct NewAbacSubjectAttr {
    pub namespace_id: Uuid,
    pub subject_id: Uuid,
    pub key: String,
    pub value: String,
}

impl From<db::abac_subject_attr::Create> for NewAbacSubjectAttr {
    fn from(msg: db::abac_subject_attr::Create) -> Self {
        NewAbacSubjectAttr {
            namespace_id: msg.namespace_id,
            subject_id: msg.subject_id,
            key: msg.key,
            value: msg.value,
        }
    }
}
