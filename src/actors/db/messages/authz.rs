use actix::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use rpc::authz::{self, Request};
use rpc::error::Result;

#[derive(Debug)]
pub struct Authz {
    pub namespace_ids: Vec<Uuid>,
    pub subject: Uuid,
    pub object: String,
    pub action: String,
}

impl Message for Authz {
    type Result = Result<bool>;
}

impl Handler<Authz> for DbExecutor {
    type Result = Result<bool>;

    fn handle(&mut self, msg: Authz, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().expect("Failed to get a connection from pool");
        authz::call(conn, &msg)
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
