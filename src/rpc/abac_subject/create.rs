use diesel::prelude::*;
use diesel::{self, PgConnection};
use uuid::Uuid;

use actors::db::abac_subject;
use models::{AbacSubjectAttr, NewAbacSubjectAttr};
use rpc::error::Result;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_id: Uuid,
    pub subject_id: Uuid,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    namespace_id: Uuid,
    subject_id: Uuid,
    key: String,
    value: String,
}

impl From<AbacSubjectAttr> for Response {
    fn from(attr: AbacSubjectAttr) -> Self {
        Response {
            namespace_id: attr.namespace_id,
            subject_id: attr.subject_id,
            key: attr.key,
            value: attr.value,
        }
    }
}

pub fn call(conn: &PgConnection, msg: abac_subject::Create) -> Result<AbacSubjectAttr> {
    use schema::abac_subject_attr::dsl::*;

    let changeset = NewAbacSubjectAttr::from(msg);
    let attr = diesel::insert_into(abac_subject_attr)
        .values(changeset)
        .get_result(conn)?;

    Ok(attr)
}
