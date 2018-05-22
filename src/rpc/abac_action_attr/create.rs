use diesel::prelude::*;
use diesel::{self, PgConnection};
use uuid::Uuid;

use actors::db::abac_action_attr;
use models::{AbacActionAttr, NewAbacActionAttr};
use rpc::error::Result;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_id: Uuid,
    pub action_id: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    namespace_id: Uuid,
    action_id: String,
    key: String,
    value: String,
}

impl From<AbacActionAttr> for Response {
    fn from(attr: AbacActionAttr) -> Self {
        Response {
            namespace_id: attr.namespace_id,
            action_id: attr.action_id,
            key: attr.key,
            value: attr.value,
        }
    }
}

pub fn call(conn: &PgConnection, msg: abac_action_attr::Create) -> Result<AbacActionAttr> {
    use schema::abac_action_attr::dsl::*;

    let changeset = NewAbacActionAttr::from(msg);
    let attr = diesel::insert_into(abac_action_attr)
        .values(changeset)
        .get_result(conn)?;

    Ok(attr)
}
