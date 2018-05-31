use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacObjectAttr;
use rpc::abac_object_attr::read;
use rpc::error::Result;

#[derive(Debug)]
pub struct Find {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Find {
    type Result = Result<AbacObjectAttr>;
}

impl Handler<Find> for DbExecutor {
    type Result = Result<AbacObjectAttr>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find {
            namespace_id: req.namespace_id,
            object_id: req.object_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Find) -> Result<AbacObjectAttr> {
    use schema::abac_object_attr::dsl::*;

    let pk = (msg.namespace_id, msg.object_id, msg.key, msg.value);
    let object = abac_object_attr.find(pk).get_result(conn)?;

    Ok(object)
}
