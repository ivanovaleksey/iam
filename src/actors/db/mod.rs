use actix::prelude::*;

pub use actors::db::messages::*;
use DbPool;

mod messages;

#[allow(missing_debug_implementations)]
pub struct DbExecutor(pub DbPool);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
