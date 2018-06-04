use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use rpc::error::Result;
use rpc::namespace::delete;

#[derive(Debug)]
pub struct Delete {
    pub id: Uuid,
}

impl Message for Delete {
    type Result = Result<Namespace>;
}

impl Handler<Delete> for DbExecutor {
    type Result = Result<Namespace>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete { id: req.id }
    }
}

fn call(conn: &PgConnection, msg: Delete) -> Result<Namespace> {
    use schema::namespace::dsl::*;

    let target = namespace.find(msg.id);
    let object = diesel::update(target)
        .set(enabled.eq(false))
        .get_result(conn)?;

    Ok(object)
}
