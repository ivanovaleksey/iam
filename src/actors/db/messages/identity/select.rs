use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::{identity::PrimaryKey, Identity};

#[derive(Debug)]
pub enum Select {
    ByIds(Vec<PrimaryKey>),
    ByAccountId(Uuid),
}

impl Message for Select {
    type Result = QueryResult<Vec<Identity>>;
}

impl Handler<Select> for DbExecutor {
    type Result = QueryResult<Vec<Identity>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        match msg {
            Select::ByIds(ids) => select_by_ids(conn, &ids),
            Select::ByAccountId(account_id) => select_by_account_id(conn, account_id),
        }
    }
}

fn select_by_ids(conn: &PgConnection, ids: &[PrimaryKey]) -> QueryResult<Vec<Identity>> {
    use diesel;
    use diesel::sql_types::Array;
    use models::identity::SqlPrimaryKey;
    use schema::identity;

    // TODO: remove it once Diesel support (provider, label, uid) IN ((), (), ()) syntax
    let query = identity::table
        .filter(
            diesel::dsl::sql(
                "array[(identity.provider, identity.label, identity.uid) :: identity_composite_pkey] <@ ",
            ).bind::<Array<SqlPrimaryKey>, _>(ids),
        )
        .order(identity::created_at.asc());

    println!(
        "<<<- QUERY: {}",
        diesel::debug_query::<diesel::pg::Pg, _>(&query)
    );

    let items = query.load(conn)?;

    Ok(items)
}

fn select_by_account_id(conn: &PgConnection, account_id: Uuid) -> QueryResult<Vec<Identity>> {
    use schema::identity;

    let query = identity::table
        .filter(identity::account_id.eq(account_id))
        .order(identity::created_at.asc());

    let items = query.load(conn)?;

    Ok(items)
}
