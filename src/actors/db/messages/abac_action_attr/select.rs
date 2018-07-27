use abac::models::AbacAction;
use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;

#[derive(Debug)]
pub struct Select {
    pub namespace_ids: Vec<Uuid>,
    pub key: Option<String>,
    pub limit: u16,
    pub offset: u16,
}

impl Message for Select {
    type Result = QueryResult<Vec<AbacAction>>;
}

impl Handler<Select> for DbExecutor {
    type Result = QueryResult<Vec<AbacAction>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, &msg)
    }
}

fn call(conn: &PgConnection, msg: &Select) -> QueryResult<Vec<AbacAction>> {
    use abac::dsl::*;
    use abac::schema::abac_action;
    use diesel::dsl::any;

    let mut query = abac_action::table
        .filter(
            abac_action::inbound
                .namespace_id()
                .eq(any(&msg.namespace_ids)),
        )
        .or_filter(
            abac_action::outbound
                .namespace_id()
                .eq(any(&msg.namespace_ids)),
        )
        .order(abac_action::created_at.asc())
        .limit(i64::from(msg.limit))
        .offset(i64::from(msg.offset))
        .into_boxed();

    if let Some(ref key) = msg.key {
        query = query
            .filter(abac_action::inbound.key().eq(key))
            .or_filter(abac_action::outbound.key().eq(key));
    }

    query.load(conn)
}
