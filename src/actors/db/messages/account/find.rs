use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Account;

#[derive(Debug)]
pub struct Find(pub Uuid);

impl Message for Find {
    type Result = QueryResult<Account>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Account>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        find_active_account(conn, msg.0)
    }
}

fn find_active_account(conn: &PgConnection, id: Uuid) -> QueryResult<Account> {
    use schema::account;

    account::table
        .filter(account::deleted_at.is_null())
        .find(id)
        .get_result(conn)
}
