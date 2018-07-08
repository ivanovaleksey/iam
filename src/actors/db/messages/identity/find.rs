use actix::prelude::*;
use diesel::prelude::*;

use actors::DbExecutor;
use models::{identity::PrimaryKey, Account, Identity};
use schema::identity;

#[derive(Debug)]
pub struct Find(pub PrimaryKey);

impl Message for Find {
    type Result = QueryResult<Identity>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Identity>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        find_identity(conn, &msg.0)
    }
}

#[derive(Debug)]
pub struct FindWithAccount(pub PrimaryKey);

impl Message for FindWithAccount {
    type Result = QueryResult<(Identity, Account)>;
}

impl Handler<FindWithAccount> for DbExecutor {
    type Result = QueryResult<(Identity, Account)>;

    fn handle(&mut self, msg: FindWithAccount, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        find_identity_with_account(conn, &msg.0)
    }
}

fn find_identity(conn: &PgConnection, pk: &PrimaryKey) -> QueryResult<Identity> {
    identity::table.find(pk.as_tuple()).get_result(conn)
}

pub fn find_identity_with_account(
    conn: &PgConnection,
    pk: &PrimaryKey,
) -> QueryResult<(Identity, Account)> {
    use schema::account;

    identity::table
        .find(pk.as_tuple())
        .inner_join(account::table)
        .get_result(conn)
}
