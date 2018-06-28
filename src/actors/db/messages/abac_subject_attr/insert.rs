use abac::{models::AbacSubject, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use rpc::abac_subject_attr::create;
use rpc::error::Result;

#[derive(Debug)]
pub struct Insert {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

impl Message for Insert {
    type Result = Result<AbacSubject>;
}

impl Handler<Insert> for DbExecutor {
    type Result = Result<AbacSubject>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<create::Request> for Insert {
    fn from(req: create::Request) -> Self {
        Insert {
            inbound: req.inbound,
            outbound: req.outbound,
        }
    }
}

fn call(conn: &PgConnection, msg: Insert) -> Result<AbacSubject> {
    use abac::schema::abac_subject::dsl::*;

    let changeset = AbacSubject {
        inbound: msg.inbound,
        outbound: msg.outbound,
    };

    let subject = diesel::insert_into(abac_subject)
        .values(changeset)
        .get_result(conn)?;

    Ok(subject)
}
