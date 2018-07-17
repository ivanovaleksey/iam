use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;

#[derive(Debug)]
pub struct Select {
    pub ids: Vec<Uuid>,
}

impl Message for Select {
    type Result = QueryResult<Vec<Namespace>>;
}

impl Handler<Select> for DbExecutor {
    type Result = QueryResult<Vec<Namespace>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

fn call(conn: &PgConnection, msg: Select) -> QueryResult<Vec<Namespace>> {
    use diesel::dsl::any;
    use schema::namespace::dsl::*;

    let query = namespace
        .filter(enabled.eq(true))
        .filter(id.eq(any(msg.ids)));

    let items = query.load(conn)?;

    Ok(items)
}
