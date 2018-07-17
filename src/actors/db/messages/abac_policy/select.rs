use abac::models::AbacPolicy;
use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use rpc::abac_policy::list;

#[derive(Debug)]
pub struct Select {
    pub namespace_ids: Vec<Uuid>,
}

impl Message for Select {
    type Result = QueryResult<Vec<AbacPolicy>>;
}

impl Handler<Select> for DbExecutor {
    type Result = QueryResult<Vec<AbacPolicy>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<list::Request> for Select {
    fn from(req: list::Request) -> Self {
        Select {
            namespace_ids: req.filter.namespace_ids,
        }
    }
}

fn call(conn: &PgConnection, msg: Select) -> QueryResult<Vec<AbacPolicy>> {
    use abac::schema::abac_policy::dsl::*;
    use diesel::dsl::any;

    let query = abac_policy.filter(namespace_id.eq(any(msg.namespace_ids)));
    let items = query.load(conn)?;

    Ok(items)
}
