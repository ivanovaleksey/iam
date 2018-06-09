use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use models::{Account, NewAccount};

#[derive(Clone, Copy, Debug)]
pub struct Insert {
    pub enabled: bool,
}

impl Message for Insert {
    type Result = QueryResult<Account>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<Account>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        insert(conn, msg)
    }
}

fn insert(conn: &PgConnection, msg: Insert) -> QueryResult<Account> {
    use schema::account::dsl::*;

    let changeset = NewAccount::from(msg);
    let attr = diesel::insert_into(account)
        .values(changeset)
        .get_result(conn)?;

    Ok(attr)
}
