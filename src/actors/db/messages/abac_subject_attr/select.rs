use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::AbacSubjectAttr;
use rpc::abac_subject_attr::list;
use rpc::error::Result;

#[derive(Debug)]
pub struct Select {
    pub namespace_id: Uuid,
    pub subject_id: Option<Uuid>,
    pub key: Option<String>,
}

impl Message for Select {
    type Result = Result<Vec<AbacSubjectAttr>>;
}

impl Handler<Select> for DbExecutor {
    type Result = Result<Vec<AbacSubjectAttr>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<list::Request> for Select {
    fn from(req: list::Request) -> Self {
        let filter = req.filter.0;
        Select {
            namespace_id: filter.namespace_id,
            subject_id: filter.subject_id,
            key: filter.key,
        }
    }
}

fn call(conn: &PgConnection, msg: Select) -> Result<Vec<AbacSubjectAttr>> {
    use schema::abac_subject_attr::dsl::*;

    let mut query = abac_subject_attr.into_boxed();

    query = query.filter(namespace_id.eq(msg.namespace_id));

    if let Some(subject) = msg.subject_id {
        query = query.filter(subject_id.eq(subject));
    }

    if let Some(k) = msg.key {
        query = query.filter(key.eq(k));
    }

    let items = query.load(conn)?;

    Ok(items)
}
