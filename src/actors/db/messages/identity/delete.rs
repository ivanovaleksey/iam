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
            Delete::Identity(ref pk) => delete_identity(conn, pk),
            Delete::IdentityWithAccount(ref pk) => delete_identity_with_account(conn, pk),
        }
    }
}

fn delete_identity(conn: &PgConnection, pk: &PrimaryKey) -> QueryResult<Identity> {
    use schema::identity;

    conn.transaction::<_, _, _>(|| {
        let target = identity::table.find(pk.as_tuple());
        let identity = diesel::delete(target).get_result(conn)?;

        delete_identity_links(conn, &identity)?;

        Ok(identity)
    })
}

fn delete_identity_with_account(conn: &PgConnection, pk: &PrimaryKey) -> QueryResult<Identity> {
    use actors::db::account;

    conn.transaction::<_, _, _>(|| {
        let identity = delete_identity(conn, &pk)?;

        account::delete::delete(conn, identity.account_id)?;

        Ok(identity)
    })
}

fn delete_identity_links(conn: &PgConnection, identity: &Identity) -> QueryResult<usize> {
    use abac::{schema::abac_object, types::AbacAttribute};
    use abac_attribute::UriKind;
    use settings;

    let pk = PrimaryKey {
        provider: identity.provider,
        label: identity.label.clone(),
        uid: identity.uid.clone(),
    };
    let iam_namespace_id = settings::iam_namespace_id();

    diesel::delete(abac_object::table.filter(
        abac_object::inbound.eq(AbacAttribute::new(iam_namespace_id, UriKind::Identity(pk))),
    )).execute(conn)
}
