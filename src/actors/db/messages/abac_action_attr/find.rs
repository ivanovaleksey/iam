use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacActionAttr;
use rpc::abac_action_attr::read;
use rpc::error::Result;

#[derive(Debug)]
pub struct Find {
    pub namespace_id: Uuid,
    pub action_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Find {
    type Result = Result<AbacActionAttr>;
}

impl Handler<Find> for DbExecutor {
    type Result = Result<AbacActionAttr>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find {
            namespace_id: req.namespace_id,
            action_id: req.action_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Find) -> Result<AbacActionAttr> {
    use schema::abac_action_attr::dsl::*;

    let pk = (msg.namespace_id, msg.action_id, msg.key, msg.value);
    let object = abac_action_attr.find(pk).get_result(conn)?;

    Ok(object)
}
