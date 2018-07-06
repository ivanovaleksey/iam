use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use models::{identity::PrimaryKey, Identity};

#[derive(Debug)]
pub enum Delete {
    Identity(PrimaryKey),
    IdentityWithAccount(PrimaryKey),
}

impl Message for Delete {
    type Result = QueryResult<Identity>;
}

impl Handler<Delete> for DbExecutor {
    type Result = QueryResult<Identity>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        match msg {
            Delete::Identity(pk) => delete_identity(conn, pk),
            Delete::IdentityWithAccount(pk) => delete_identity_with_account(conn, pk),
        }
    }
}

fn delete_identity(conn: &PgConnection, pk: PrimaryKey) -> QueryResult<Identity> {
    use schema::identity;

    let target = identity::table.find(pk.as_tuple());
    let identity = diesel::delete(target).get_result(conn)?;

    Ok(identity)
}

fn delete_identity_with_account(conn: &PgConnection, pk: PrimaryKey) -> QueryResult<Identity> {
    use actors::db::account;

    conn.transaction::<_, _, _>(|| {
        let identity = delete_identity(conn, pk)?;

        account::delete::delete(conn, identity.account_id)?;

        Ok(identity)
    })
}
