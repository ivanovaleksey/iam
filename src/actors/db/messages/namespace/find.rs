use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use rpc::namespace::read;
use schema::namespace;

#[derive(Debug)]
pub enum Find {
    Any(Uuid),
    Enabled(Uuid),
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
            Find::Enabled(id) => find_enabled(conn, id),
        }
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find::Enabled(req.id)
    }
}

fn find_any(conn: &PgConnection, id: Uuid) -> QueryResult<Namespace> {
    namespace::table.find(id).get_result(conn)
}

fn find_enabled(conn: &PgConnection, id: Uuid) -> QueryResult<Namespace> {
    namespace::table
        .filter(namespace::enabled.eq(true))
        .find(id)
        .get_result(conn)
}
