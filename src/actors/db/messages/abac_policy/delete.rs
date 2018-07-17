use abac::{models::AbacPolicy, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use rpc::abac_policy::delete;

#[derive(Debug)]
pub struct Delete {
    pub namespace_id: Uuid,
    pub subject: Vec<AbacAttribute>,
    pub object: Vec<AbacAttribute>,
    pub action: Vec<AbacAttribute>,
}

impl Message for Delete {
    type Result = QueryResult<AbacPolicy>;
}

impl Handler<Delete> for DbExecutor {
    type Result = QueryResult<AbacPolicy>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete {
            namespace_id: req.namespace_id,
            subject: req.subject,
            object: req.object,
            action: req.action,
        }
    }
}

fn call(conn: &PgConnection, msg: Delete) -> QueryResult<AbacPolicy> {
    use abac::schema::abac_policy::dsl::*;

    let pk = (msg.subject, msg.object, msg.action, msg.namespace_id);
    let target = abac_policy.find(pk);
    let policy = diesel::delete(target).get_result(conn)?;

    Ok(policy)
}
