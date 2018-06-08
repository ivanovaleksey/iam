use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacPolicy;
use rpc::abac_policy::delete;
use rpc::error::Result;

#[derive(Debug)]
pub struct Delete {
    pub namespace_id: Uuid,
    pub subject_namespace_id: Uuid,
    pub subject_key: String,
    pub subject_value: String,
    pub object_namespace_id: Uuid,
    pub object_key: String,
    pub object_value: String,
    pub action_namespace_id: Uuid,
    pub action_key: String,
    pub action_value: String,
}

impl Message for Delete {
    type Result = Result<AbacPolicy>;
}

impl Handler<Delete> for DbExecutor {
    type Result = Result<AbacPolicy>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete {
            namespace_id: req.namespace_id,
            subject_namespace_id: req.subject_namespace_id,
            subject_key: req.subject_key,
            subject_value: req.subject_value,
            object_namespace_id: req.object_namespace_id,
            object_key: req.object_key,
            object_value: req.object_value,
            action_namespace_id: req.action_namespace_id,
            action_key: req.action_key,
            action_value: req.action_value,
        }
    }
}

fn call(conn: &PgConnection, msg: Delete) -> Result<AbacPolicy> {
    use schema::abac_policy::dsl::*;

    let pk = (
        msg.namespace_id,
        msg.subject_namespace_id,
        msg.subject_key,
        msg.subject_value,
        msg.object_namespace_id,
        msg.object_key,
        msg.object_value,
        msg.action_namespace_id,
        msg.action_key,
        msg.action_value,
    );
    let target = abac_policy.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
