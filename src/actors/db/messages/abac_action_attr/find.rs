use abac::{models::AbacAction, AbacAttribute};
use actix::prelude::*;
use diesel::prelude::*;

use actors::DbExecutor;
use rpc::abac_action_attr::read;

#[derive(Debug)]
pub struct Find {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

impl Message for Find {
    type Result = QueryResult<AbacAction>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<AbacAction>;

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

fn call(conn: &PgConnection, msg: Find) -> QueryResult<AbacAction> {
    use abac::schema::abac_action::dsl::*;

    let pk = (msg.inbound, msg.outbound);
    let action = abac_action.find(pk).get_result(conn)?;

    Ok(action)
}
