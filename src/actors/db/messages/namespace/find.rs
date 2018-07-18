use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use schema::namespace;

#[derive(Debug)]
pub enum Find {
    Any(Uuid),
    Active(Uuid),
}

impl Message for Find {
    type Result = QueryResult<Namespace>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Namespace>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        match msg {
            Find::Any(id) => find_any(conn, id),
            Find::Active(id) => find_enabled(conn, id),
        }
    }
}

fn find_any(conn: &PgConnection, id: Uuid) -> QueryResult<Namespace> {
    namespace::table.find(id).get_result(conn)
}

fn find_enabled(conn: &PgConnection, id: Uuid) -> QueryResult<Namespace> {
    namespace::table
        .filter(namespace::deleted_at.is_null())
        .find(id)
        .get_result(conn)
}
