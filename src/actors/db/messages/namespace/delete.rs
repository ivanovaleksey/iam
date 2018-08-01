use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::Namespace;
use rpc::namespace::delete;

#[derive(Debug)]
pub struct Delete {
    pub id: Uuid,
}

impl Message for Delete {
    type Result = QueryResult<Namespace>;
}

impl Handler<Delete> for DbExecutor {
    type Result = QueryResult<Namespace>;

    fn handle(&mut self, msg: Delete, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        delete_namespace(conn, msg.id)
    }
}

impl From<delete::Request> for Delete {
    fn from(req: delete::Request) -> Self {
        Delete { id: req.id }
    }
}

fn delete_namespace(conn: &PgConnection, id: Uuid) -> QueryResult<Namespace> {
    use schema::namespace;

    conn.transaction::<_, _, _>(|| {
        let target = namespace::table.find(id);
        let record = diesel::update(target)
            .set(namespace::deleted_at.eq(diesel::dsl::now))
            .get_result(conn)?;

        delete_namespace_links(conn, &record)?;

        Ok(record)
    })
}

fn delete_namespace_links(conn: &PgConnection, namespace: &Namespace) -> QueryResult<usize> {
    use abac::{schema::abac_object, AbacAttribute};
    use abac_attribute::UriKind;
    use settings;

    let iam_namespace_id = settings::iam_namespace_id();

    diesel::delete(
        abac_object::table.filter(abac_object::inbound.eq(AbacAttribute::new(
            iam_namespace_id,
            UriKind::Namespace(namespace.id),
        ))),
    ).execute(conn)
}
