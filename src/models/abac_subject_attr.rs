use uuid::Uuid;

use models::Namespace;
use schema::abac_subject_attr;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace)]
#[primary_key(namespace_id, value, subject_id)]
#[table_name = "abac_subject_attr"]
pub struct AbacSubjectAttr {
    pub namespace_id: Uuid,
    pub subject_id: Uuid,
    pub value: String,
}
