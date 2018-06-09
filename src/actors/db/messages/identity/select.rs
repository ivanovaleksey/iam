use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Identity;
use rpc::error::Result;
use rpc::identity::list;

#[derive(Clone, Copy, Debug)]
pub struct Select {
    pub provider: Option<Uuid>,
    pub account_id: Option<Uuid>,
}

impl Message for Select {
    type Result = Result<Vec<Identity>>;
}

impl Handler<Select> for DbExecutor {
    type Result = Result<Vec<Identity>>;

    fn handle(&mut self, msg: Select, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<list::Request> for Select {
    fn from(req: list::Request) -> Self {
        let filter = req.filter.0;
        Select {
            provider: filter.provider,
            account_id: filter.account_id,
        }
    }
}

fn call(conn: &PgConnection, msg: Select) -> Result<Vec<Identity>> {
    use schema::identity::dsl::*;

    let mut query = identity.into_boxed();

    if let Some(value) = msg.provider {
        query = query.filter(provider.eq(value));
    }

    if let Some(value) = msg.account_id {
        query = query.filter(account_id.eq(value));
    }

    let items = query.load(conn)?;

    Ok(items)
}
