use abac::{models::AbacObject, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use rpc::abac_object_attr::create;
use rpc::error::Result;

#[derive(Debug)]
pub struct Insert {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

impl Message for Insert {
    type Result = Result<AbacObject>;
}

impl Handler<Insert> for DbExecutor {
    type Result = Result<AbacObject>;

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

fn call(conn: &PgConnection, msg: Insert) -> Result<AbacObject> {
    use abac::schema::abac_object::dsl::*;

    let changeset = AbacObject {
        inbound: msg.inbound,
        outbound: msg.outbound,
    };

    let object = diesel::insert_into(abac_object)
        .values(changeset)
        .get_result(conn)?;

    Ok(object)
}
