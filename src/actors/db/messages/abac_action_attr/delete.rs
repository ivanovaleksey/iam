use abac::{models::AbacAction, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use rpc::abac_action_attr::delete;

#[derive(Debug)]
pub struct Delete {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

impl Message for Delete {
    type Result = QueryResult<AbacAction>;
}

impl Handler<Delete> for DbExecutor {
    type Result = QueryResult<AbacAction>;

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

fn call(conn: &PgConnection, msg: Delete) -> QueryResult<AbacAction> {
    use abac::schema::abac_action::dsl::*;

    let pk = (msg.inbound, msg.outbound);
    let target = abac_action.find(pk);
    let action = diesel::delete(target).get_result(conn)?;

    Ok(action)
}
