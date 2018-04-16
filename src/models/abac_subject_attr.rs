use uuid::Uuid;

use models::namespace::Namespace;
use schema::abac_subject_attr;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace)]
#[primary_key(namespace_id, value, subject_id)]
#[table_name = "abac_subject_attr"]
pub struct AbacSubjectAttr {
    namespace_id: Uuid,
    subject_id: String,
    value: String,
}
