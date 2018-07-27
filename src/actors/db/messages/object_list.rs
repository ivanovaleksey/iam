use abac::AbacAttribute;
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;

#[derive(Debug)]
pub struct ObjectList {
    pub objects: Vec<AbacAttribute>,
    pub limit: u16,
    pub offset: u16,
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

    let query = diesel::select(abac_object_list(
        &msg.objects,
        i32::from(msg.offset),
        i32::from(msg.limit),
    ));

    query.get_results(conn)
}
