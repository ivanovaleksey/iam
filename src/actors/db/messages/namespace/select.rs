use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use rpc::error::Result;
use rpc::namespace::list;

#[derive(Clone, Copy, Debug)]
pub struct Select {
    pub account_id: Uuid,
}

impl Message for Select {
    type Result = Result<Vec<Namespace>>;
}

impl Handler<Select> for DbExecutor {
    type Result = Result<Vec<Namespace>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<list::Request> for Select {
    fn from(req: list::Request) -> Self {
        let filter = req.filter.0;
        Select {
            account_id: filter.account_id,
        }
    }
}

fn call(conn: &PgConnection, msg: Select) -> Result<Vec<Namespace>> {
    use schema::namespace::dsl::*;

    let query = namespace
        .filter(enabled.eq(true))
        .filter(account_id.eq(msg.account_id));

    let items = query.load(conn)?;

    Ok(items)
}
