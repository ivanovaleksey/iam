use actix::prelude::*;
use diesel::prelude::*;

use actors::DbExecutor;
use models::{NewRefreshToken, RefreshToken};

#[derive(Debug)]
pub struct Update(pub NewRefreshToken);

impl Message for Update {
    type Result = QueryResult<RefreshToken>;
}

impl Handler<Update> for DbExecutor {
    type Result = QueryResult<RefreshToken>;

    fn handle(&mut self, msg: Update, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        update_token(conn, msg.0)
    }
}

fn update_token(conn: &PgConnection, changeset: NewRefreshToken) -> QueryResult<RefreshToken> {
    let token = changeset.save_changes(conn)?;
    Ok(token)
}
