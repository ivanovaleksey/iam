use diesel::prelude::*;
use diesel::{self, PgConnection};

use actors::db::abac_action;
use models::AbacActionAttr;
use rpc;
use rpc::error::Result;

pub type Request = rpc::abac_action::create::Request;
pub type Response = rpc::abac_action::create::Response;

pub fn call(conn: &PgConnection, msg: abac_action::Delete) -> Result<AbacActionAttr> {
    use schema::abac_action_attr::dsl::*;

    let pk = (msg.namespace_id, msg.action_id, msg.key, msg.value);
    let target = abac_action_attr.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
