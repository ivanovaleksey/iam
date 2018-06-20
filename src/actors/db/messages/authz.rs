use abac::types::AbacAttribute;
use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use rpc::authz::Request;

#[derive(Debug)]
pub struct Authz {
    pub namespace_ids: Vec<Uuid>,
    pub subject: Vec<AbacAttribute>,
    pub object: Vec<AbacAttribute>,
    pub action: Vec<AbacAttribute>,
}

impl Message for Authz {
    type Result = QueryResult<bool>;
}

impl Handler<Authz> for DbExecutor {
    type Result = QueryResult<bool>;

    fn handle(&mut self, msg: Authz, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().expect("Failed to get a connection from pool");
        call(conn, &msg)
    }
}

impl From<Request> for Authz {
    fn from(req: Request) -> Self {
        Authz {
            namespace_ids: req.namespace_ids,
            subject: req.subject,
            object: req.object,
            action: req.action,
        }
    }
}

fn call(conn: &PgConnection, msg: &Authz) -> QueryResult<bool> {
    use abac::functions::abac_authorize;
    let granted = diesel::select(abac_authorize(
        &msg.subject,
        &msg.object,
        &msg.action,
        &msg.namespace_ids,
    )).get_result(conn)?;

    Ok(granted)
}
