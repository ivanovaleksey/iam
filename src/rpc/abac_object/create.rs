use diesel::prelude::*;
use diesel::{self, PgConnection};
use uuid::Uuid;

use actors::db::abac_object;
use models::{AbacObjectAttr, NewAbacObjectAttr};
use rpc::error::Result;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    namespace_id: Uuid,
    object_id: String,
    key: String,
    value: String,
}

impl From<AbacObjectAttr> for Response {
    fn from(object: AbacObjectAttr) -> Self {
        Response {
            namespace_id: object.namespace_id,
            object_id: object.object_id,
            key: object.key,
            value: object.value,
        }
    }
}

pub fn call(conn: &PgConnection, msg: abac_object::Create) -> Result<AbacObjectAttr> {
    use schema::abac_object_attr::dsl::*;

    let changeset = NewAbacObjectAttr::from(msg);
    let object = diesel::insert_into(abac_object_attr)
        .values(changeset)
        .get_result(conn)?;

    Ok(object)
}
