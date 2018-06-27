use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::{identity::PrimaryKey, Account, Identity, NewAccount, NewIdentity};

#[derive(Debug)]
pub enum Insert {
    Identity { pk: PrimaryKey, account_id: Uuid },
    IdentityWithAccount(PrimaryKey),
}

impl Message for Insert {
    type Result = QueryResult<Identity>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<Identity>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        match msg {
            Insert::Identity { pk, account_id } => insert_identity(conn, pk, account_id),
            Insert::IdentityWithAccount(pk) => insert_identity_with_account(conn, pk),
        }
    }
}

fn insert_identity(conn: &PgConnection, pk: PrimaryKey, account_id: Uuid) -> QueryResult<Identity> {
    use schema::identity;

    let changeset = NewIdentity {
        provider: pk.provider,
        label: pk.label,
        uid: pk.uid,
        account_id,
    };
    let identity = diesel::insert_into(identity::table)
        .values(changeset)
        .get_result(conn)?;

    Ok(identity)
}

fn insert_identity_with_account(conn: &PgConnection, pk: PrimaryKey) -> QueryResult<Identity> {
    use schema::account;

    conn.transaction::<_, _, _>(|| {
        let account = diesel::insert_into(account::table)
            .values(NewAccount { enabled: true })
            .get_result::<Account>(conn)?;

        let identity = insert_identity(conn, pk, account.id)?;

        Ok(identity)
    })
}
