use abac::{models::AbacAction, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use rpc::abac_action_attr::create;

#[derive(Debug)]
pub struct Insert {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

impl Message for Insert {
    type Result = QueryResult<AbacAction>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<AbacAction>;

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

fn call(conn: &PgConnection, msg: Insert) -> QueryResult<AbacAction> {
    use abac::schema::abac_action::dsl::*;

    let changeset = AbacAction {
        inbound: msg.inbound,
        outbound: msg.outbound,
    };

    let action = diesel::insert_into(abac_action)
        .values(changeset)
        .get_result(conn)?;

    Ok(action)
}
