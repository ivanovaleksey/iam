use actix::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use rpc::auth::{self, Request};
use rpc::error::Result;

#[derive(Debug)]
pub struct Auth {
    pub namespace_ids: Vec<Uuid>,
    pub subject: Uuid,
    pub object: String,
    pub action: String,
}

impl Message for Auth {
    type Result = Result<bool>;
}

impl Handler<Auth> for DbExecutor {
    type Result = Result<bool>;

    fn handle(&mut self, msg: Auth, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().expect("Failed to get a connection from pool");
        auth::call(conn, &msg)
    }
}

impl From<Request> for Auth {
    fn from(req: Request) -> Self {
        Auth {
            namespace_ids: req.namespace_ids,
            subject: req.subject,
            object: req.object,
            action: req.action,
        }
    }
}
