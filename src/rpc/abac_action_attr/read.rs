use diesel::prelude::*;
use diesel::PgConnection;

use actors::db::abac_action_attr;
use models::AbacActionAttr;
use rpc;
use rpc::error::Result;

pub type Request = rpc::abac_action_attr::create::Request;
pub type Response = rpc::abac_action_attr::create::Response;

pub fn call(conn: &PgConnection, msg: abac_action_attr::Read) -> Result<AbacActionAttr> {
    use schema::abac_action_attr::dsl::*;

    let pk = (msg.namespace_id, msg.action_id, msg.key, msg.value);
    let object = abac_action_attr.find(pk).get_result(conn)?;

    Ok(object)
}
