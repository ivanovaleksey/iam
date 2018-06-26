use abac::types::AbacAttribute;
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;

#[derive(Debug)]
pub struct ObjectList {
    pub objects: Vec<AbacAttribute>,
}

impl Message for ObjectList {
    type Result = QueryResult<Vec<AbacAttribute>>;
}

impl Handler<ObjectList> for DbExecutor {
    type Result = QueryResult<Vec<AbacAttribute>>;

    fn handle(&mut self, msg: ObjectList, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().expect("Failed to get a connection from pool");
        call(conn, &msg)
    }
}

fn call(conn: &PgConnection, msg: &ObjectList) -> QueryResult<Vec<AbacAttribute>> {
    use abac::functions::abac_object_list;

    let granted = diesel::select(abac_object_list(&msg.objects, 0, 100)).get_results(conn)?;

    Ok(granted)
}
