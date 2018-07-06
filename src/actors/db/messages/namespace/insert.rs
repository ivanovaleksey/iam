use abac::{models::AbacObject, schema::abac_object, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use models::{Namespace, NewNamespace};

#[derive(Debug)]
pub struct Insert(pub NewNamespace);

impl Message for Insert {
    type Result = QueryResult<Namespace>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<Namespace>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        insert_namespace(conn, msg.0)
    }
}

fn insert_namespace(conn: &PgConnection, changeset: NewNamespace) -> QueryResult<Namespace> {
    use schema::namespace;

    conn.transaction::<_, _, _>(|| {
        let namespace = diesel::insert_into(namespace::table)
            .values(changeset)
            .get_result::<Namespace>(conn)?;

        insert_namespace_links(conn, &namespace)?;

        Ok(namespace)
    })
}

pub fn insert_namespace_links(conn: &PgConnection, namespace: &Namespace) -> QueryResult<usize> {
    use settings;

    let iam_namespace_id = settings::iam_namespace_id();

    diesel::insert_into(abac_object::table)
        .values(vec![
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: iam_namespace_id,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", namespace.id),
                },
                outbound: AbacAttribute {
                    namespace_id: iam_namespace_id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", namespace.account_id),
                },
            },
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: iam_namespace_id,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", namespace.id),
                },
                outbound: AbacAttribute {
                    namespace_id: iam_namespace_id,
                    key: "type".to_owned(),
                    value: "namespace".to_owned(),
                },
            },
        ])
        .execute(conn)
}
