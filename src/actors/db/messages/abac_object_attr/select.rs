use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacObjectAttr;
use rpc::abac_object_attr::list;
use rpc::error::Result;

#[derive(Debug)]
pub struct Select {
    pub namespace_id: Uuid,
    pub object_id: Option<String>,
    pub key: Option<String>,
}

impl Message for Select {
    type Result = Result<Vec<AbacObjectAttr>>;
}

impl Handler<Select> for DbExecutor {
    type Result = Result<Vec<AbacObjectAttr>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<list::Request> for Select {
    fn from(req: list::Request) -> Self {
        let filter = req.filter.0;
        Select {
            namespace_id: filter.namespace_id,
            object_id: filter.object_id,
            key: filter.key,
        }
    }
}

fn call(conn: &PgConnection, msg: Select) -> Result<Vec<AbacObjectAttr>> {
    use schema::abac_object_attr::dsl::*;

    let mut query = abac_object_attr.into_boxed();

    query = query.filter(namespace_id.eq(msg.namespace_id));

    if let Some(object) = msg.object_id {
        query = query.filter(object_id.eq(object));
    }

    if let Some(k) = msg.key {
        query = query.filter(key.eq(k));
    }

    let items = query.load(conn)?;

    Ok(items)
}
