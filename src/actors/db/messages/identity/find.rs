use actix::prelude::*;
use diesel::prelude::*;

use actors::DbExecutor;
use models::{identity::PrimaryKey, Identity};

#[derive(Debug)]
pub struct Find(pub PrimaryKey);

impl Message for Find {
    type Result = QueryResult<Identity>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Identity>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, &msg.0)
    }
}

fn call(conn: &PgConnection, pk: &PrimaryKey) -> QueryResult<Identity> {
    use schema::identity;

    identity::table.find(pk.as_tuple()).get_result(conn)
}
