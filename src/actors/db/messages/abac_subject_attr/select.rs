use abac::models::AbacSubject;
use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;

#[derive(Debug)]
pub struct Select {
    pub namespace_ids: Vec<Uuid>,
    pub key: Option<String>,
}

impl Message for Select {
    type Result = QueryResult<Vec<AbacSubject>>;
}

impl Handler<Select> for DbExecutor {
    type Result = QueryResult<Vec<AbacSubject>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, &msg)
    }
}

fn call(conn: &PgConnection, msg: &Select) -> QueryResult<Vec<AbacSubject>> {
    use abac::dsl::*;
    use abac::schema::abac_subject;
    use diesel::dsl::any;

    let mut query = abac_subject::table
        .filter(
            abac_subject::inbound
                .namespace_id()
                .eq(any(&msg.namespace_ids)),
        )
        .or_filter(
            abac_subject::outbound
                .namespace_id()
                .eq(any(&msg.namespace_ids)),
        )
        .into_boxed();

    if let Some(ref key) = msg.key {
        query = query
            .filter(abac_subject::inbound.key().eq(key))
            .or_filter(abac_subject::outbound.key().eq(key));
    }

    query.load(conn)
}
