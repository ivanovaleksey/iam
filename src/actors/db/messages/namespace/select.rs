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
        call(conn, &msg.ids)
    }
}

fn call(conn: &PgConnection, ids: &[Uuid]) -> QueryResult<Vec<Namespace>> {
    use diesel::dsl::any;
    use schema::namespace;

    let query = namespace::table
        .filter(namespace::deleted_at.is_null())
        .filter(namespace::id.eq(any(ids)))
        .order(namespace::created_at.asc());

    query.load(conn)
}
