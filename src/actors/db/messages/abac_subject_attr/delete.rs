use abac::{models::AbacSubject, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use rpc::abac_subject_attr::delete;
use rpc::error::Result;

#[derive(Debug)]
pub struct Delete {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

impl Message for Delete {
    type Result = Result<AbacSubject>;
}

impl Handler<Delete> for DbExecutor {
    type Result = Result<AbacSubject>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete {
            inbound: req.inbound,
            outbound: req.outbound,
        }
    }
}

fn call(conn: &PgConnection, msg: Delete) -> Result<AbacSubject> {
    use abac::schema::abac_subject::dsl::*;

    let pk = (msg.inbound, msg.outbound);
    let target = abac_subject.find(pk);
    let subject = diesel::delete(target).get_result(conn)?;

    Ok(subject)
}
