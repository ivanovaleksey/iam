use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use rpc::authz::Request;
use schema::{abac_action_attr, abac_object_attr, abac_policy, abac_subject_attr};

#[derive(Debug)]
pub struct Authz {
    pub namespace_ids: Vec<Uuid>,
    pub subject: Uuid,
    pub object: String,
    pub action: String,
}

impl Authz {
    pub fn execute_namespace_message(namespace_id: Uuid, subject_id: Uuid) -> Self {
        Authz {
            namespace_ids: vec![namespace_id],
            subject: subject_id,
            object: format!("namespace.{}", namespace_id),
            action: "execute".to_owned(),
        }
    }
}

impl Message for Authz {
    type Result = QueryResult<bool>;
}

impl Handler<Authz> for DbExecutor {
    type Result = QueryResult<bool>;

    fn handle(&mut self, msg: Authz, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().expect("Failed to get a connection from pool");
        call(conn, &msg)
    }
}

impl From<Request> for Authz {
    fn from(req: Request) -> Self {
        Authz {
            namespace_ids: req.namespace_ids,
            subject: req.subject,
            object: req.object,
            action: req.action,
        }
    }
}

fn call(conn: &PgConnection, msg: &Authz) -> QueryResult<bool> {
    let query = diesel::select(diesel::dsl::exists(
        abac_policy::table
            .inner_join(
                abac_subject_attr::table.on(abac_policy::subject_namespace_id
                    .eq(abac_subject_attr::namespace_id)
                    .and(abac_policy::subject_key.eq(abac_subject_attr::key))
                    .and(abac_policy::subject_value.eq(abac_subject_attr::value))),
            )
            .inner_join(
                abac_object_attr::table.on(abac_policy::object_namespace_id
                    .eq(abac_object_attr::namespace_id)
                    .and(abac_policy::object_key.eq(abac_object_attr::key))
                    .and(abac_policy::object_value.eq(abac_object_attr::value))),
            )
            .inner_join(
                abac_action_attr::table.on(abac_policy::action_namespace_id
                    .eq(abac_action_attr::namespace_id)
                    .and(abac_policy::action_key.eq(abac_action_attr::key))
                    .and(abac_policy::action_value.eq(abac_action_attr::value))),
            )
            .filter(
                abac_policy::not_before
                    .is_null()
                    .or(abac_policy::not_before.lt(diesel::dsl::now.nullable())),
            )
            .filter(
                abac_policy::expired_at
                    .is_null()
                    .or(abac_policy::expired_at.gt(diesel::dsl::now.nullable())),
            )
            .filter(abac_policy::namespace_id.eq_any(&msg.namespace_ids))
            .filter(abac_subject_attr::subject_id.eq(&msg.subject))
            .filter(abac_object_attr::object_id.eq(&msg.object))
            .filter(abac_action_attr::action_id.eq(&msg.action))
            .select(abac_policy::all_columns)
            .limit(1),
    ));

    let res = query.get_result(conn)?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::dsl::IntervalDsl;

    use std::env;

    use models::*;
    use schema::*;

    fn establish_connection() -> PgConnection {
        let database_url = env::var("DATABASE_URL").unwrap();
        PgConnection::establish(&database_url).unwrap()
    }

    fn prepare_attributes(conn: &PgConnection) -> (Namespace, AbacSubjectAttr) {
        let account = diesel::insert_into(account::table)
            .values(account::enabled.eq(true))
            .get_result::<Account>(conn)
            .unwrap();

        let namespace = diesel::insert_into(namespace::table)
            .values((
                namespace::label.eq("example.org"),
                namespace::account_id.eq(account.id),
                namespace::enabled.eq(true),
            ))
            .get_result::<Namespace>(conn)
            .unwrap();

        let subject_attr = diesel::insert_into(abac_subject_attr::table)
            .values((
                abac_subject_attr::namespace_id.eq(namespace.id),
                abac_subject_attr::subject_id.eq(account.id),
                abac_subject_attr::key.eq("role".to_owned()),
                abac_subject_attr::value.eq("client".to_owned()),
            ))
            .get_result::<AbacSubjectAttr>(conn)
            .unwrap();

        diesel::insert_into(abac_object_attr::table)
            .values((
                abac_object_attr::namespace_id.eq(namespace.id),
                abac_object_attr::object_id.eq("room"),
                abac_object_attr::key.eq("type"),
                abac_object_attr::value.eq("room"),
            ))
            .execute(conn)
            .unwrap();

        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("create"),
                abac_action_attr::key.eq("access"),
                abac_action_attr::value.eq("*"),
            ))
            .execute(conn)
            .unwrap();
        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("read"),
                abac_action_attr::key.eq("access"),
                abac_action_attr::value.eq("*"),
            ))
            .execute(conn)
            .unwrap();

        (namespace, subject_attr)
    }

    #[test]
    fn test_allowed_action() {
        let conn = establish_connection();
        conn.begin_test_transaction().unwrap();

        let (namespace, subject_attr) = prepare_attributes(&conn);

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("room"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("access"),
                abac_policy::action_value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();

        let msg = Authz {
            namespace_ids: vec![namespace.id],
            subject: subject_attr.subject_id,
            object: "room".to_owned(),
            action: "create".to_owned(),
        };
        assert_eq!(Ok(true), call(&conn, &msg));
    }

    #[test]
    fn test_denied_action() {
        let conn = establish_connection();
        conn.begin_test_transaction().unwrap();

        let (namespace, subject_attr) = prepare_attributes(&conn);

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("room"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("access"),
                abac_policy::action_value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();

        let msg = Authz {
            namespace_ids: vec![namespace.id],
            subject: subject_attr.subject_id,
            object: "room".to_owned(),
            action: "delete".to_owned(),
        };
        assert_eq!(Ok(false), call(&conn, &msg));
    }

    #[test]
    fn test_with_another_namespace() {
        let conn = establish_connection();
        conn.begin_test_transaction().unwrap();

        let (namespace, subject_attr) = prepare_attributes(&conn);

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("room"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("access"),
                abac_policy::action_value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();

        let another_namespace = Uuid::new_v4();

        let msg = Authz {
            namespace_ids: vec![namespace.id, another_namespace],
            subject: subject_attr.subject_id,
            object: "room".to_owned(),
            action: "create".to_owned(),
        };
        assert_eq!(Ok(true), call(&conn, &msg));

        let msg = Authz {
            namespace_ids: vec![another_namespace],
            subject: subject_attr.subject_id,
            object: "room".to_owned(),
            action: "create".to_owned(),
        };
        assert_eq!(Ok(false), call(&conn, &msg));
    }

    #[test]
    fn test_before_activation() {
        let conn = establish_connection();
        conn.begin_test_transaction().unwrap();

        let (namespace, subject_attr) = prepare_attributes(&conn);

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("room"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("access"),
                abac_policy::action_value.eq("*"),
                abac_policy::not_before.eq((diesel::dsl::now + 1.day()).nullable()),
            ))
            .execute(&conn)
            .unwrap();

        let msg = Authz {
            namespace_ids: vec![namespace.id],
            subject: subject_attr.subject_id,
            object: "room".to_owned(),
            action: "create".to_owned(),
        };

        assert_eq!(Ok(false), call(&conn, &msg));
    }

    #[test]
    fn test_after_activation() {
        let conn = establish_connection();
        conn.begin_test_transaction().unwrap();

        let (namespace, subject_attr) = prepare_attributes(&conn);

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("room"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("access"),
                abac_policy::action_value.eq("*"),
                abac_policy::not_before.eq((diesel::dsl::now - 1.hour()).nullable()),
            ))
            .execute(&conn)
            .unwrap();

        let msg = Authz {
            namespace_ids: vec![namespace.id],
            subject: subject_attr.subject_id,
            object: "room".to_owned(),
            action: "create".to_owned(),
        };

        assert_eq!(Ok(true), call(&conn, &msg));
    }

    #[test]
    fn test_expiration() {
        let conn = establish_connection();
        conn.begin_test_transaction().unwrap();

        let (namespace, subject_attr) = prepare_attributes(&conn);

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("room"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("access"),
                abac_policy::action_value.eq("*"),
                abac_policy::expired_at.eq((diesel::dsl::now - 1.hour()).nullable()),
            ))
            .execute(&conn)
            .unwrap();

        let msg = Authz {
            namespace_ids: vec![namespace.id],
            subject: subject_attr.subject_id,
            object: "room".to_owned(),
            action: "create".to_owned(),
        };

        assert_eq!(Ok(false), call(&conn, &msg));
    }

    #[test]
    fn test_after_expiration() {
        let conn = establish_connection();
        conn.begin_test_transaction().unwrap();

        let (namespace, subject_attr) = prepare_attributes(&conn);

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("room"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("access"),
                abac_policy::action_value.eq("*"),
                abac_policy::expired_at.eq((diesel::dsl::now + 1.day()).nullable()),
            ))
            .execute(&conn)
            .unwrap();

        let msg = Authz {
            namespace_ids: vec![namespace.id],
            subject: subject_attr.subject_id,
            object: "room".to_owned(),
            action: "create".to_owned(),
        };

        assert_eq!(Ok(true), call(&conn, &msg));
    }
}
