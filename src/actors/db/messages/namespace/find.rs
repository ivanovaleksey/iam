use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use rpc::namespace::read;

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
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find::Enabled(req.id)
    }
}

fn call(conn: &PgConnection, msg: Find) -> QueryResult<Namespace> {
    use schema::namespace;

    let record = match msg {
        Find::Any(id) => namespace::table.find(id).get_result(conn)?,
        Find::Enabled(id) => namespace::table
            .filter(namespace::enabled.eq(true))
            .find(id)
            .get_result(conn)?,
    };

    Ok(record)
}
