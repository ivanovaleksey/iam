use diesel::prelude::*;
use diesel::{self, PgConnection};

use actors::db::abac_subject;
use models::AbacSubjectAttr;
use rpc;
use rpc::error::Result;

pub type Request = rpc::abac_subject::create::Request;
pub type Response = rpc::abac_subject::create::Response;

pub fn call(conn: &PgConnection, msg: abac_subject::Delete) -> Result<AbacSubjectAttr> {
    use schema::abac_subject_attr::dsl::*;

    let pk = (msg.namespace_id, msg.subject_id, msg.key, msg.value);
    let target = abac_subject_attr.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}