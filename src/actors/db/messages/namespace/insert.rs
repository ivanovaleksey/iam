use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::{Namespace, NewNamespace};
use rpc::error::Result;
use rpc::namespace::create;

#[derive(Debug)]
pub struct Insert {
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
}

impl Message for Insert {
    type Result = Result<Namespace>;
}

impl Handler<Insert> for DbExecutor {
    type Result = Result<Namespace>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<create::Request> for Insert {
    fn from(req: create::Request) -> Self {
        Insert {
            label: req.label,
            account_id: req.account_id,
            enabled: req.enabled,
        }
    }
}

fn call(conn: &PgConnection, msg: Insert) -> Result<Namespace> {
    use schema::namespace::dsl::*;

    let changeset = NewNamespace::from(msg);
    let attr = diesel::insert_into(namespace)
        .values(changeset)
        .get_result(conn)?;

    Ok(attr)
}
