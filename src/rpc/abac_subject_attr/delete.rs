use diesel::prelude::*;
use diesel::{self, PgConnection};

use actors::db::abac_subject_attr;
use models::AbacSubjectAttr;
use rpc::{self, error::Result};

pub type Request = rpc::abac_subject_attr::create::Request;
pub type Response = rpc::abac_subject_attr::create::Response;

pub fn call(conn: &PgConnection, msg: abac_subject_attr::Delete) -> Result<AbacSubjectAttr> {
    use schema::abac_subject_attr::dsl::*;

    let pk = (msg.namespace_id, msg.subject_id, msg.key, msg.value);
    let target = abac_subject_attr.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
