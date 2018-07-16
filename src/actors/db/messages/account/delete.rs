use abac::{
    schema::{abac_object, abac_policy}, types::AbacAttribute,
};
use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use abac_attribute::UriKind;
use actors::DbExecutor;
use models::Account;
use settings;

#[derive(Debug)]
pub struct Delete {
    pub id: Uuid,
}

impl Message for Delete {
    type Result = QueryResult<Account>;
}

impl Handler<Delete> for DbExecutor {
    type Result = QueryResult<Account>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        delete(conn, msg.id)
    }
}

pub fn delete(conn: &PgConnection, id: Uuid) -> QueryResult<Account> {
    use schema::account;

    conn.transaction::<_, _, _>(|| {
        let target = account::table.find(id);
        let account = diesel::delete(target).get_result::<Account>(conn)?;

        delete_account_links(conn, account.id)?;
        delete_account_policies(conn, account.id)?;

        Ok(account)
    })
}

fn delete_account_links(conn: &PgConnection, id: Uuid) -> QueryResult<usize> {
    let iam_namespace_id = settings::iam_namespace_id();

    diesel::delete(abac_object::table.filter(
        abac_object::inbound.eq(AbacAttribute::new(iam_namespace_id, UriKind::Account(id))),
    )).execute(conn)
}

fn delete_account_policies(conn: &PgConnection, id: Uuid) -> QueryResult<usize> {
    let iam_namespace_id = settings::iam_namespace_id();

    diesel::delete(
        abac_policy::table.filter(abac_policy::subject.eq(vec![AbacAttribute::new(
            iam_namespace_id,
            UriKind::Account(id),
        )])),
    ).execute(conn)
}
