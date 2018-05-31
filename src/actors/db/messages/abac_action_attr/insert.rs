use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::{AbacActionAttr, NewAbacActionAttr};
use rpc::abac_action_attr::create;
use rpc::error::Result;

#[derive(Debug)]
pub struct Insert {
    pub namespace_id: Uuid,
    pub action_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Insert {
    type Result = Result<AbacActionAttr>;
}

impl Handler<Insert> for DbExecutor {
    type Result = Result<AbacActionAttr>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<create::Request> for Insert {
    fn from(req: create::Request) -> Self {
        Insert {
            namespace_id: req.namespace_id,
            action_id: req.action_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Insert) -> Result<AbacActionAttr> {
    use schema::abac_action_attr::dsl::*;

    let changeset = NewAbacActionAttr::from(msg);
    let attr = diesel::insert_into(abac_action_attr)
        .values(changeset)
        .get_result(conn)?;

    Ok(attr)
}
