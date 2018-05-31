use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacSubjectAttr;
use rpc::abac_subject_attr::delete;
use rpc::error::Result;

#[derive(Debug)]
pub struct Delete {
    pub namespace_id: Uuid,
    pub subject_id: Uuid,
    pub key: String,
    pub value: String,
}

impl Message for Delete {
    type Result = Result<AbacSubjectAttr>;
}

impl Handler<Delete> for DbExecutor {
    type Result = Result<AbacSubjectAttr>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete {
            namespace_id: req.namespace_id,
            subject_id: req.subject_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Delete) -> Result<AbacSubjectAttr> {
    use schema::abac_subject_attr::dsl::*;

    let pk = (msg.namespace_id, msg.subject_id, msg.key, msg.value);
    let target = abac_subject_attr.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
