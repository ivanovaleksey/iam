use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacPolicy;
use rpc::abac_policy::list;
use rpc::error::Result;

#[derive(Debug)]
pub struct Select {
    pub namespace_id: Uuid,
}

impl Message for Select {
    type Result = Result<Vec<AbacPolicy>>;
}

impl Handler<Select> for DbExecutor {
    type Result = Result<Vec<AbacPolicy>>;

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
        }
    }
}

fn call(conn: &PgConnection, msg: Select) -> Result<Vec<AbacPolicy>> {
    use schema::abac_policy::dsl::*;

    let query = abac_policy.filter(namespace_id.eq(msg.namespace_id));

    let items = query.load(conn)?;

    Ok(items)
}
