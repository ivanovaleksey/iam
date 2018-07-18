use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Account;
use rpc::account::read;

#[derive(Debug)]
pub struct Find {
    pub id: Uuid,
}

impl Message for Find {
    type Result = QueryResult<Account>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Account>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg.id)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find { id: req.id }
    }
}

fn call(conn: &PgConnection, id: Uuid) -> QueryResult<Account> {
    use schema::account;

    account::table.find(id).get_result(conn)
}
