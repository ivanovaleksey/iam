use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use schema::namespace;

#[derive(Debug)]
pub enum Find {
    Any(Uuid),
    Active(Uuid),
    ByLabel(String),
}

impl Message for Find {
    type Result = QueryResult<Namespace>;
}

impl Handler<Find> for DbExecutor {
    type Result = QueryResult<Namespace>;

    fn handle(&mut self, msg: Find, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        match msg {
            Find::Any(id) => find_any(conn, id),
            Find::Active(id) => find_active(conn, id),
            Find::ByLabel(ref label) => find_by_label(conn, label),
        }
    }
}

fn find_any(conn: &PgConnection, id: Uuid) -> QueryResult<Namespace> {
    namespace::table.find(id).get_result(conn)
}

fn find_active(conn: &PgConnection, id: Uuid) -> QueryResult<Namespace> {
    namespace::table
        .filter(namespace::deleted_at.is_null())
        .find(id)
        .get_result(conn)
}

fn find_by_label(conn: &PgConnection, label: &str) -> QueryResult<Namespace> {
    namespace::table
        .filter(namespace::deleted_at.is_null())
        .filter(namespace::label.eq(label))
        .first(conn)
}
