use abac::{models::AbacPolicy, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use rpc::abac_policy::create;
use rpc::error::Result;

#[derive(Debug)]
pub struct Insert {
    pub namespace_id: Uuid,
    pub subject: Vec<AbacAttribute>,
    pub object: Vec<AbacAttribute>,
    pub action: Vec<AbacAttribute>,
}

impl Message for Insert {
    type Result = Result<AbacPolicy>;
}

impl Handler<Insert> for DbExecutor {
    type Result = Result<AbacPolicy>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<create::Request> for Insert {
    fn from(req: create::Request) -> Self {
        Insert {
            namespace_id: req.namespace_id,
            subject: req.subject,
            object: req.object,
            action: req.action,
        }
    }
}

fn call(conn: &PgConnection, msg: Insert) -> Result<AbacPolicy> {
    use abac::schema::abac_policy::dsl::*;

    let changeset = AbacPolicy {
        namespace_id: msg.namespace_id,
        subject: msg.subject,
        object: msg.object,
        action: msg.action,
    };
    let policy = diesel::insert_into(abac_policy)
        .values(changeset)
        .get_result(conn)?;

    Ok(policy)
}
