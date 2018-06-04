use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::Identity;
use rpc::error::Result;
use rpc::identity::delete;

#[derive(Debug)]
pub struct Delete {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
}

impl Message for Delete {
    type Result = Result<Identity>;
}

impl Handler<Delete> for DbExecutor {
    type Result = Result<Identity>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete {
            provider: req.provider,
            label: req.label,
            uid: req.uid,
        }
    }
}

fn call(conn: &PgConnection, msg: Delete) -> Result<Identity> {
    use schema::identity::dsl::*;

    let pk = (msg.provider, msg.label, msg.uid);
    let target = identity.find(pk);
    let object = diesel::delete(target).get_result(conn)?;

    Ok(object)
}
