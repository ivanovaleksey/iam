use uuid::Uuid;

use actors::db::abac_subject;
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
    namespace_id: Uuid,
    subject_id: Uuid,
    key: String,
    value: String,
}

impl From<abac_subject::Create> for NewAbacSubjectAttr {
    fn from(msg: abac_subject::Create) -> Self {
        NewAbacSubjectAttr {
            namespace_id: msg.namespace_id,
            subject_id: msg.subject_id,
            key: msg.key,
            value: msg.value,
        }
    }
}
