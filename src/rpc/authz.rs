use diesel::prelude::*;
use diesel::{self, PgConnection};
use futures::Future;
use jsonrpc::{self, BoxFuture};
use uuid::Uuid;

use actors::db::authz::Authz;
use rpc;
use rpc::error::Result;
use schema::{abac_action_attr, abac_object_attr, abac_policy, abac_subject_attr};

build_rpc_trait! {
    pub trait Rpc {
        type Metadata;

        #[rpc(meta, name = "authorize")]
        fn authz(&self, Self::Metadata, Request) -> BoxFuture<Response>;
    }
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub namespace_ids: Vec<Uuid>,
    pub subject: Uuid,
    pub object: String,
    pub action: String,
}

#[derive(Debug, Serialize)]
pub struct Response(bool);

impl Response {
    pub fn new(value: bool) -> Self {
        Response(value)
    }
}

pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn authz(&self, meta: rpc::Meta, req: Request) -> BoxFuture<Response> {
        let msg = Authz::from(req);
        let fut = meta.db
            .unwrap()
            .send(msg)
            .map_err(|_| jsonrpc::Error::internal_error())
            .and_then(|res| match res {
                Ok(res) => Ok(Response::new(res)),
                Err(e) => Err(e.into()),
            });

        Box::new(fut)
    }
}

pub fn call(conn: &PgConnection, msg: &Authz) -> Result<bool> {
    let query = diesel::select(diesel::dsl::exists(
        abac_policy::table
            .inner_join(
                abac_subject_attr::table.on(abac_policy::subject_value
                    .eq(abac_subject_attr::value)
                    .and(abac_policy::namespace_id.eq(abac_subject_attr::namespace_id))),
            )
            .inner_join(
                abac_object_attr::table.on(abac_policy::object_value
                    .eq(abac_object_attr::value)
                    .and(abac_policy::namespace_id.eq(abac_object_attr::namespace_id))),
            )
            .inner_join(
                abac_action_attr::table.on(abac_policy::action_value
                    .eq(abac_action_attr::value)
                    .and(abac_policy::namespace_id.eq(abac_action_attr::namespace_id))),
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
            .select(abac_policy::id)
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
                abac_action_attr::value.eq("access:owner"),
            ))
            .execute(conn)
            .unwrap();
        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("read"),
                abac_action_attr::value.eq("access:owner"),
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
                abac_policy::subject_value.eq("role:client"),
                abac_policy::object_value.eq("type:room"),
                abac_policy::action_value.eq("access:owner"),
                abac_policy::issued_at.eq(diesel::dsl::now),
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
                abac_policy::subject_value.eq("role:client"),
                abac_policy::object_value.eq("type:room"),
                abac_policy::action_value.eq("access:owner"),
                abac_policy::issued_at.eq(diesel::dsl::now),
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
                abac_policy::subject_value.eq("role:client"),
                abac_policy::object_value.eq("type:room"),
                abac_policy::action_value.eq("access:owner"),
                abac_policy::issued_at.eq(diesel::dsl::now),
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
                abac_policy::subject_value.eq("role:client"),
                abac_policy::object_value.eq("type:room"),
                abac_policy::action_value.eq("access:owner"),
                abac_policy::issued_at.eq(diesel::dsl::now),
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
                abac_policy::subject_value.eq("role:client"),
                abac_policy::object_value.eq("type:room"),
                abac_policy::action_value.eq("access:owner"),
                abac_policy::issued_at.eq(diesel::dsl::now),
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
                abac_policy::subject_value.eq("role:client"),
                abac_policy::object_value.eq("type:room"),
                abac_policy::action_value.eq("access:owner"),
                abac_policy::issued_at.eq(diesel::dsl::now),
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
                abac_policy::subject_value.eq("role:client"),
                abac_policy::object_value.eq("type:room"),
                abac_policy::action_value.eq("access:owner"),
                abac_policy::issued_at.eq(diesel::dsl::now),
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
