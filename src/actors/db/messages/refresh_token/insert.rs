use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use models::{NewRefreshToken, RefreshToken};
use schema::refresh_token;

#[derive(Debug)]
pub struct Insert(pub NewRefreshToken);

impl Message for Insert {
    type Result = QueryResult<RefreshToken>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<RefreshToken>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        insert_token(conn, msg.0)
    }
}

pub fn insert_token(conn: &PgConnection, changeset: NewRefreshToken) -> QueryResult<RefreshToken> {
    diesel::insert_into(refresh_token::table)
        .values(changeset)
        .get_result(conn)
}
