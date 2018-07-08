use actix::prelude::*;
use diesel::prelude::*;

use actors::DbExecutor;
use models::{identity::PrimaryKey, Account, Identity, RefreshToken};

#[derive(Debug)]
pub struct Upsert(pub PrimaryKey);

impl Message for Upsert {
    type Result = QueryResult<(Identity, Account, RefreshToken)>;
}

impl Handler<Upsert> for DbExecutor {
    type Result = QueryResult<(Identity, Account, RefreshToken)>;

    fn handle(&mut self, msg: Upsert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        upsert_identity(conn, msg.0)
    }
}

fn upsert_identity(
    conn: &PgConnection,
    pk: PrimaryKey,
) -> QueryResult<(Identity, Account, RefreshToken)> {
    use actors::db;
    use schema::refresh_token;

    let existing = db::identity::find::find_identity_with_account(conn, &pk).optional()?;

    if let Some((identity, account)) = existing {
        let token = refresh_token::table.find(account.id).get_result(conn)?;

        Ok((identity, account, token))
    } else {
        db::identity::insert::insert_identity_with_account(conn, pk)
    }
}
