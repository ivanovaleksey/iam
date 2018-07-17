use abac::{models::AbacSubject, types::AbacAttribute};
use actix::prelude::*;
use diesel::prelude::*;

use actors::DbExecutor;
use rpc::abac_subject_attr::read;

#[derive(Debug)]
pub struct Find {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

impl Message for Find {
    type Result = QueryResult<AbacSubject>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<AbacSubject>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<read::Request> for Find {
    fn from(req: read::Request) -> Self {
        Find {
            inbound: req.inbound,
            outbound: req.outbound,
        }
    }
}

fn call(conn: &PgConnection, msg: Find) -> QueryResult<AbacSubject> {
    use abac::schema::abac_subject::dsl::*;

    let pk = (msg.inbound, msg.outbound);
    let subject = abac_subject.find(pk).get_result(conn)?;

    Ok(subject)
}
