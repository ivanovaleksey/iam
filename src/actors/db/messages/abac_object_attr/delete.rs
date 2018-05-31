use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacObjectAttr;
use rpc::abac_object_attr::delete;
use rpc::error::Result;

#[derive(Debug)]
pub struct Delete {
    pub namespace_id: Uuid,
    pub object_id: String,
    pub key: String,
    pub value: String,
}

impl Message for Delete {
    type Result = Result<AbacObjectAttr>;
}

impl Handler<Delete> for DbExecutor {
    type Result = Result<AbacObjectAttr>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete {
            namespace_id: req.namespace_id,
            object_id: req.object_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Delete) -> Result<AbacObjectAttr> {
    use schema::abac_object_attr::dsl::*;

    let pk = (msg.namespace_id, msg.object_id, msg.key, msg.value);
    let target = abac_object_attr.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
