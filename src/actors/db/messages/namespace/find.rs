use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use rpc::error::Result;
use rpc::namespace::read;

#[derive(Debug)]
pub struct Find {
    pub id: Uuid,
}

impl Message for Find {
    type Result = Result<Namespace>;
}

impl Handler<Find> for DbExecutor {
    type Result = Result<Namespace>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find { id: req.id }
    }
}

fn call(conn: &PgConnection, msg: Find) -> Result<Namespace> {
    use schema::namespace::dsl::*;

    let object = namespace
        .filter(enabled.eq(true))
        .find(msg.id)
        .get_result(conn)?;

    Ok(object)
}
