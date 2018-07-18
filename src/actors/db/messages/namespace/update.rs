use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use rpc::namespace::update;
use schema::namespace;

#[derive(Debug, AsChangeset, Identifiable)]
#[table_name = "namespace"]
pub struct Update {
    pub id: Uuid,
    pub label: Option<String>,
    pub enabled: Option<bool>,
}

impl Message for Update {
    type Result = QueryResult<Namespace>;
}

impl Handler<Update> for DbExecutor {
    type Result = QueryResult<Namespace>;

    fn handle(&mut self, msg: Update, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, &msg)
    }
}

impl From<update::Request> for Update {
    fn from(req: update::Request) -> Self {
        Update {
            id: req.id,
            label: req.label,
            enabled: req.enabled,
        }
    }
}

fn call(conn: &PgConnection, msg: &Update) -> QueryResult<Namespace> {
    msg.save_changes(conn)
}
