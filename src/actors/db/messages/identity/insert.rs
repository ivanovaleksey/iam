use abac::{models::AbacObject, schema::abac_object, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use models::{identity::PrimaryKey, Identity, NewAccount, NewIdentity};

#[derive(Debug)]
pub enum Insert {
    Identity(NewIdentity),
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
            Insert::Identity(changeset) => insert_identity(conn, changeset),
            Insert::IdentityWithAccount(pk) => insert_identity_with_account(conn, pk),
        }
    }
}

fn insert_identity(conn: &PgConnection, changeset: NewIdentity) -> QueryResult<Identity> {
    use schema::identity;

    conn.transaction::<_, _, _>(|| {
        let identity = diesel::insert_into(identity::table)
            .values(changeset)
            .get_result(conn)?;

        insert_identity_links(conn, &identity)?;

        Ok(identity)
    })
}

fn insert_identity_with_account(conn: &PgConnection, pk: PrimaryKey) -> QueryResult<Identity> {
    use actors::db::account;

    conn.transaction::<_, _, _>(|| {
        let account = account::insert::insert_account(conn, NewAccount { enabled: true })?;

        let changeset = NewIdentity {
            provider: pk.provider,
            label: pk.label,
            uid: pk.uid,
            account_id: account.id,
        };
        let identity = insert_identity(conn, changeset)?;

        Ok(identity)
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
