use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use rpc::error::Result;

#[derive(Debug)]
pub struct Select {
    pub ids: Vec<Uuid>,
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

fn call(conn: &PgConnection, msg: Select) -> Result<Vec<Namespace>> {
    use diesel::dsl::any;
    use schema::namespace::dsl::*;

    let query = namespace
        .filter(enabled.eq(true))
        .filter(id.eq(any(msg.ids)));

    let items = query.load(conn)?;

    Ok(items)
}
