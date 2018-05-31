use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacActionAttr;
use rpc::abac_action_attr::list;
use rpc::error::Result;

#[derive(Debug)]
pub struct Select {
    pub namespace_id: Uuid,
    pub action_id: Option<String>,
    pub key: Option<String>,
}

impl Message for Select {
    type Result = Result<Vec<AbacActionAttr>>;
}

impl Handler<Select> for DbExecutor {
    type Result = Result<Vec<AbacActionAttr>>;

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
            action_id: filter.action_id,
            key: filter.key,
        }
    }
}

fn call(conn: &PgConnection, msg: Select) -> Result<Vec<AbacActionAttr>> {
    use schema::abac_action_attr::dsl::*;

    let mut query = abac_action_attr.into_boxed();

    query = query.filter(namespace_id.eq(msg.namespace_id));

    if let Some(action) = msg.action_id {
        query = query.filter(action_id.eq(action));
    }

    if let Some(k) = msg.key {
        query = query.filter(key.eq(k));
    }

    let items = query.load(conn)?;

    Ok(items)
}
