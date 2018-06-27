use actix::prelude::*;
use diesel::prelude::*;

use actors::DbExecutor;
use models::{identity::PrimaryKey, Identity};
use rpc::identity::read;

#[derive(Debug)]
pub struct Find(pub PrimaryKey);

impl Message for Find {
    type Result = QueryResult<Identity>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Identity>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        let pk = PrimaryKey {
            provider: req.provider,
            label: req.label,
            uid: req.uid,
        };

        Find(pk)
    }
}

fn call(conn: &PgConnection, msg: Find) -> QueryResult<Identity> {
    use schema::identity::dsl::*;

    identity.find(msg.0.as_tuple()).get_result(conn)
}
