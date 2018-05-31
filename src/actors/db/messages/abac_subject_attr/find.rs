use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacSubjectAttr;
use rpc::abac_subject_attr::read;
use rpc::error::Result;

#[derive(Debug)]
pub struct Find {
    pub namespace_id: Uuid,
    pub subject_id: Uuid,
    pub key: String,
    pub value: String,
}

impl Message for Find {
    type Result = Result<AbacSubjectAttr>;
}

impl Handler<Find> for DbExecutor {
    type Result = Result<AbacSubjectAttr>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find {
            namespace_id: req.namespace_id,
            subject_id: req.subject_id,
            key: req.key,
            value: req.value,
        }
    }
}

fn call(conn: &PgConnection, msg: Find) -> Result<AbacSubjectAttr> {
    use schema::abac_subject_attr::dsl::*;

    let pk = (msg.namespace_id, msg.subject_id, msg.key, msg.value);
    let object = abac_subject_attr.find(pk).get_result(conn)?;

    Ok(object)
}
