use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::{AbacSubjectAttr, NewAbacSubjectAttr};
use rpc::abac_subject_attr::create;
use rpc::error::Result;

#[derive(Debug)]
pub struct Insert {
    pub namespace_id: Uuid,
    pub subject_id: Uuid,
    pub key: String,
    pub value: String,
}

impl Message for Insert {
    type Result = Result<AbacSubjectAttr>;
}

impl Handler<Insert> for DbExecutor {
    type Result = Result<AbacSubjectAttr>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<create::Request> for Insert {
    fn from(req: create::Request) -> Self {
        Insert {
            namespace_id: req.namespace_id,
            subject_id: req.subject_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Insert) -> Result<AbacSubjectAttr> {
    use schema::abac_subject_attr::dsl::*;

    let changeset = NewAbacSubjectAttr::from(msg);
    let object = diesel::insert_into(abac_subject_attr)
        .values(changeset)
        .get_result(conn)?;

    Ok(object)
}
