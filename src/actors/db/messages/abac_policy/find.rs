use abac::{models::AbacPolicy, types::AbacAttribute};
use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use rpc::abac_policy::read;
use rpc::error::Result;

#[derive(Debug)]
pub struct Find {
    pub namespace_id: Uuid,
    pub subject: Vec<AbacAttribute>,
    pub object: Vec<AbacAttribute>,
    pub action: Vec<AbacAttribute>,
}

impl Message for Find {
    type Result = Result<AbacPolicy>;
}

impl Handler<Find> for DbExecutor {
    type Result = Result<AbacPolicy>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find {
            namespace_id: req.namespace_id,
            subject: req.subject,
            object: req.object,
            action: req.action,
        }
    }
}

fn call(conn: &PgConnection, msg: Find) -> Result<AbacPolicy> {
    use abac::schema::abac_policy::dsl::*;

    let pk = (msg.subject, msg.object, msg.action, msg.namespace_id);
    let policy = abac_policy.find(pk).get_result(conn)?;

    Ok(policy)
}
