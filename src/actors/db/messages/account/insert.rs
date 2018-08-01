use abac::{
    models::{NewAbacObject, NewAbacPolicy}, schema::{abac_object, abac_policy}, AbacAttribute,
};
use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use abac_attribute::{CollectionKind, OperationKind, UriKind};
use actors::DbExecutor;
use models::Account;
use settings;

#[derive(Debug)]
pub struct Insert;

impl Message for Insert {
    type Result = QueryResult<Account>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<Account>;

    fn handle(&mut self, _msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        insert_account(conn)
    }
}

pub fn insert_account(conn: &PgConnection) -> QueryResult<Account> {
    use schema::account;

    conn.transaction::<_, _, _>(|| {
        let account = diesel::insert_into(account::table)
            .default_values()
            .get_result::<Account>(conn)?;

        insert_account_links(conn, account.id)?;
        insert_account_policies(conn, account.id)?;

        Ok(account)
    })
}

pub fn insert_account_links(conn: &PgConnection, account_id: Uuid) -> QueryResult<usize> {
    let iam_namespace_id = settings::iam_namespace_id();

    let account_uri = UriKind::Account(account_id);
    diesel::insert_into(abac_object::table)
        .values(vec![
            NewAbacObject {
                inbound: AbacAttribute::new(iam_namespace_id, account_uri.clone()),
                outbound: AbacAttribute::new(iam_namespace_id, CollectionKind::Account),
            },
            NewAbacObject {
                inbound: AbacAttribute::new(iam_namespace_id, account_uri),
                outbound: AbacAttribute::new(
                    iam_namespace_id,
                    UriKind::Namespace(iam_namespace_id),
                ),
            },
        ])
        .execute(conn)
}

pub fn insert_account_policies(conn: &PgConnection, account_id: Uuid) -> QueryResult<usize> {
    let iam_namespace_id = settings::iam_namespace_id();

    let account_uri = UriKind::Account(account_id);
    diesel::insert_into(abac_policy::table)
        .values(NewAbacPolicy {
            subject: vec![AbacAttribute::new(iam_namespace_id, account_uri.clone())],
            object: vec![AbacAttribute::new(iam_namespace_id, account_uri)],
            action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Any)],
            namespace_id: iam_namespace_id,
        })
        .execute(conn)
}
