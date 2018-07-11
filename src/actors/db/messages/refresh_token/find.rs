use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::{Account, RefreshToken};
use schema::refresh_token;

#[derive(Debug)]
pub struct FindWithAccount(pub Uuid);

impl Message for FindWithAccount {
    type Result = QueryResult<(RefreshToken, Account)>;
}

impl Handler<FindWithAccount> for DbExecutor {
    type Result = QueryResult<(RefreshToken, Account)>;

    fn handle(&mut self, msg: FindWithAccount, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        find_token_with_account(conn, msg.0)
    }
}

fn find_token_with_account(
    conn: &PgConnection,
    account_id: Uuid,
) -> QueryResult<(RefreshToken, Account)> {
    use schema::account;

    refresh_token::table
        .find(account_id)
        .inner_join(account::table)
        .get_result(conn)
}
