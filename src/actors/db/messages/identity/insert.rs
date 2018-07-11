use abac::{models::AbacObject, schema::abac_object, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use models::{identity::PrimaryKey, Account, Identity, NewIdentity, NewRefreshToken, RefreshToken};

#[derive(Debug)]
pub struct Insert(pub NewIdentity);

impl Message for Insert {
    type Result = QueryResult<Identity>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<Identity>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        insert_identity(conn, &msg.0)
    }
}

#[derive(Debug)]
pub struct InsertWithAccount(pub PrimaryKey);

impl Message for InsertWithAccount {
    type Result = QueryResult<(Identity, Account, RefreshToken)>;
}

impl Handler<InsertWithAccount> for DbExecutor {
    type Result = QueryResult<(Identity, Account, RefreshToken)>;

    fn handle(&mut self, msg: InsertWithAccount, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        insert_identity_with_account(conn, msg.0)
    }
}

fn insert_identity(conn: &PgConnection, changeset: &NewIdentity) -> QueryResult<Identity> {
    use schema::identity;

    conn.transaction::<_, _, _>(|| {
        let identity = diesel::insert_into(identity::table)
            .values(changeset)
            .get_result(conn)?;

        insert_identity_links(conn, &identity)?;

        Ok(identity)
    })
}

pub fn insert_identity_with_account(
    conn: &PgConnection,
    pk: PrimaryKey,
) -> QueryResult<(Identity, Account, RefreshToken)> {
    use actors::db;

    conn.transaction::<_, _, _>(|| {
        let account = db::account::insert::insert_account(conn)?;

        // TODO: do not unwrap
        let changeset = NewRefreshToken::try_new(account.id).unwrap();
        let token = db::refresh_token::insert::insert_token(conn, changeset)?;

        let changeset = NewIdentity {
            provider: pk.provider,
            label: pk.label,
            uid: pk.uid,
            account_id: account.id,
        };
        let identity = insert_identity(conn, &changeset)?;

        Ok((identity, account, token))
    })
}

pub fn insert_identity_links(conn: &PgConnection, identity: &Identity) -> QueryResult<usize> {
    use abac_attribute::{CollectionKind, UriKind};
    use settings;

    let pk = PrimaryKey {
        provider: identity.provider,
        label: identity.label.clone(),
        uid: identity.uid.clone(),
    };
    let iam_namespace_id = settings::iam_namespace_id();

    let identity_uri = UriKind::Identity(pk);
    diesel::insert_into(abac_object::table)
        .values(vec![
            AbacObject {
                inbound: AbacAttribute::new(iam_namespace_id, identity_uri.clone()),
                outbound: AbacAttribute::new(iam_namespace_id, CollectionKind::Identity),
            },
            AbacObject {
                inbound: AbacAttribute::new(iam_namespace_id, identity_uri.clone()),
                outbound: AbacAttribute::new(
                    iam_namespace_id,
                    UriKind::Account(identity.account_id),
                ),
            },
            AbacObject {
                inbound: AbacAttribute::new(iam_namespace_id, identity_uri),
                outbound: AbacAttribute::new(
                    iam_namespace_id,
                    UriKind::Namespace(identity.provider),
                ),
            },
        ])
        .execute(conn)
}
