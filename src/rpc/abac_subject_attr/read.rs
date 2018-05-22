use diesel::prelude::*;
use diesel::PgConnection;

use actors::db::abac_subject_attr;
use models::AbacSubjectAttr;
use rpc;
use rpc::error::Result;

pub type Request = rpc::abac_subject_attr::create::Request;
pub type Response = rpc::abac_subject_attr::create::Response;

pub fn call(conn: &PgConnection, msg: abac_subject_attr::Read) -> Result<AbacSubjectAttr> {
    use schema::abac_subject_attr::dsl::*;

    let pk = (msg.namespace_id, msg.subject_id, msg.key, msg.value);
    let object = abac_subject_attr.find(pk).get_result(conn)?;

    Ok(object)
}