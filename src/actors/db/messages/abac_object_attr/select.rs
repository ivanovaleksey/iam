use abac::models::AbacObject;
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
    type Result = QueryResult<Vec<AbacObject>>;
}

impl Handler<Select> for DbExecutor {
    type Result = QueryResult<Vec<AbacObject>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        select(conn, &msg)
    }
}

fn select(conn: &PgConnection, msg: &Select) -> QueryResult<Vec<AbacObject>> {
    use abac::dsl::*;
    use abac::schema::abac_object;
    use diesel::dsl::any;

    let mut query = abac_object::table
        .filter(
            abac_object::inbound
                .namespace_id()
                .eq(any(&msg.namespace_ids)),
        )
        .or_filter(
            abac_object::outbound
                .namespace_id()
                .eq(any(&msg.namespace_ids)),
        )
        .into_boxed();

    if let Some(ref key) = msg.key {
        query = query
            .filter(abac_object::inbound.key().eq(key))
            .or_filter(abac_object::outbound.key().eq(key));
    }

    query.load(conn)
}
