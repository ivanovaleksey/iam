use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Identity;
use rpc::identity::read;

#[derive(Debug)]
pub struct Find {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
}

impl Message for Find {
    type Result = QueryResult<Identity>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Identity>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find {
            provider: req.provider,
            label: req.label,
            uid: req.uid,
        }
    }
}

fn call(conn: &PgConnection, msg: Find) -> QueryResult<Identity> {
    use schema::identity::dsl::*;

    let pk = (msg.provider, msg.label, msg.uid);
    identity.find(pk).get_result(conn)
}
