use actix::prelude::*;
use chrono::NaiveDateTime;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::{AbacPolicy, NewAbacPolicy};
use rpc::abac_policy::create;
use rpc::error::Result;

#[derive(Debug)]
pub struct Insert {
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
    pub not_before: Option<NaiveDateTime>,
    pub expired_at: Option<NaiveDateTime>,
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
            subject_namespace_id: req.subject_namespace_id,
            subject_key: req.subject_key,
            subject_value: req.subject_value,
            object_namespace_id: req.object_namespace_id,
            object_key: req.object_key,
            object_value: req.object_value,
            action_namespace_id: req.action_namespace_id,
            action_key: req.action_key,
            action_value: req.action_value,
            not_before: req.not_before,
            expired_at: req.expired_at,
        }
    }
}

fn call(conn: &PgConnection, msg: Insert) -> Result<AbacPolicy> {
    use schema::abac_policy::dsl::*;

    let changeset = NewAbacPolicy::from(msg);
    let attr = diesel::insert_into(abac_policy)
        .values(changeset)
        .get_result(conn)?;

    Ok(attr)
}
