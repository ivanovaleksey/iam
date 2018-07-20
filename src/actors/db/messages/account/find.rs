use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Account;
use schema::account;

#[derive(Debug)]
pub enum Find {
    Any(Uuid),
    Active(Uuid),
    Enabled(Uuid),
}

impl Message for Find {
    type Result = QueryResult<Account>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Account>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        match msg {
            Find::Any(id) => find_account(&conn, id),
            Find::Active(id) => find_active_account(&conn, id),
            Find::Enabled(id) => find_enabled_account(&conn, id),
        }
    }
}

fn find_account(conn: &PgConnection, id: Uuid) -> QueryResult<Account> {
    account::table.find(id).get_result(conn)
}

fn find_active_account(conn: &PgConnection, id: Uuid) -> QueryResult<Account> {
    account::table
        .filter(account::deleted_at.is_null())
        .find(id)
        .get_result(conn)
}

fn find_enabled_account(conn: &PgConnection, id: Uuid) -> QueryResult<Account> {
    account::table
        .filter(account::disabled_at.is_null())
        .filter(account::deleted_at.is_null())
        .find(id)
        .get_result(conn)
}
