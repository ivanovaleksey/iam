use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::{Identity, NewIdentity};

#[derive(Debug)]
pub struct Insert {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
    pub account_id: Uuid,
}

impl Message for Insert {
    type Result = QueryResult<Identity>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<Identity>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

fn call(conn: &PgConnection, msg: Insert) -> QueryResult<Identity> {
    use schema::identity::dsl::*;

    let changeset = NewIdentity::from(msg);
    let attr = diesel::insert_into(identity)
        .values(changeset)
        .get_result(conn)?;

    Ok(attr)
}
