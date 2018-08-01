use abac::{models::AbacObject, AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use rpc::abac_object_attr::delete;

#[derive(Debug)]
pub struct Delete {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

impl Message for Delete {
    type Result = QueryResult<AbacObject>;
}

impl Handler<Delete> for DbExecutor {
    type Result = QueryResult<AbacObject>;

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

fn call(conn: &PgConnection, msg: Delete) -> QueryResult<AbacObject> {
    use abac::schema::abac_object::dsl::*;

    let pk = (msg.inbound, msg.outbound);
    let target = abac_object.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
