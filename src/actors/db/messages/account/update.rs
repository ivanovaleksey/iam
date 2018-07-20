use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::Account;
use schema::account;

pub use self::disable::Disable;

mod disable {
    use super::*;

    #[derive(Debug)]
    pub struct Disable(pub Uuid);

    impl Message for Disable {
        type Result = QueryResult<Account>;
    }

    impl Handler<Disable> for DbExecutor {
        type Result = QueryResult<Account>;

        fn handle(&mut self, msg: Disable, _ctx: &mut Self::Context) -> Self::Result {
            let conn = &self.0.get().unwrap();
            disable_account(conn, msg.0)
        }
    }

    fn disable_account(conn: &PgConnection, id: Uuid) -> QueryResult<Account> {
        diesel::update(account::table.find(id))
            .set(account::disabled_at.eq(diesel::dsl::now))
            .get_result(conn)
    }
}
