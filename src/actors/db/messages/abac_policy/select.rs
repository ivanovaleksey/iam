use abac::models::AbacPolicy;
use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;

#[derive(Debug)]
pub struct Select {
    pub namespace_ids: Vec<Uuid>,
    pub limit: u16,
    pub offset: u16,
}

impl Message for Select {
    type Result = QueryResult<Vec<AbacPolicy>>;
}

impl Handler<Select> for DbExecutor {
    type Result = QueryResult<Vec<AbacPolicy>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        select(conn, &msg)
    }
}

fn select(conn: &PgConnection, msg: &Select) -> QueryResult<Vec<AbacPolicy>> {
    use abac::schema::abac_policy;
    use diesel::dsl::any;

    let query = abac_policy::table
        .filter(abac_policy::namespace_id.eq(any(&msg.namespace_ids)))
        .order(abac_policy::created_at.asc())
        .limit(i64::from(msg.limit))
        .offset(i64::from(msg.offset));

    query.load(conn)
}
