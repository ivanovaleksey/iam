use diesel::prelude::*;
use diesel::{self, PgConnection};

use actors::db::abac_object_attr;
use models::AbacObjectAttr;
use rpc;
use rpc::error::Result;

pub type Request = rpc::abac_object_attr::create::Request;
pub type Response = rpc::abac_object_attr::create::Response;

pub fn call(conn: &PgConnection, msg: abac_object_attr::Delete) -> Result<AbacObjectAttr> {
    use schema::abac_object_attr::dsl::*;

    let pk = (msg.namespace_id, msg.object_id, msg.key, msg.value);
    let target = abac_object_attr.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
