use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::{AbacObjectAttr, NewAbacObjectAttr};
use rpc::abac_object_attr::create;
use rpc::error::Result;

#[derive(Debug)]
pub struct Insert {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Insert {
    type Result = Result<AbacObjectAttr>;
}

impl Handler<Insert> for DbExecutor {
    type Result = Result<AbacObjectAttr>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<create::Request> for Insert {
    fn from(req: create::Request) -> Self {
        Insert {
            namespace_id: req.namespace_id,
            object_id: req.object_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Insert) -> Result<AbacObjectAttr> {
    use schema::abac_object_attr::dsl::*;

    let changeset = NewAbacObjectAttr::from(msg);
    let object = diesel::insert_into(abac_object_attr)
        .values(changeset)
        .get_result(conn)?;

    Ok(object)
}
