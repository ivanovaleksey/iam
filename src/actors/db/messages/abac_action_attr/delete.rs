use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacActionAttr;
use rpc::abac_action_attr::delete;
use rpc::error::Result;

#[derive(Debug)]
pub struct Delete {
    pub namespace_id: Uuid,
    pub action_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Delete {
    type Result = Result<AbacActionAttr>;
}

impl Handler<Delete> for DbExecutor {
    type Result = Result<AbacActionAttr>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete {
            namespace_id: req.namespace_id,
            action_id: req.action_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Delete) -> Result<AbacActionAttr> {
    use schema::abac_action_attr::dsl::*;

    let pk = (msg.namespace_id, msg.action_id, msg.key, msg.value);
    let target = abac_action_attr.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
