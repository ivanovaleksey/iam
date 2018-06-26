use diesel::{self, prelude::*};
use jsonrpc;
use serde_json;
use uuid::Uuid;

use abac::models::{AbacObject, AbacPolicy};
use abac::schema::{abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::models::{Account, Namespace, NewNamespace};
use iam::schema::namespace;

use shared::{self, FOXFORD_ACCOUNT_ID, IAM_ACCOUNT_ID};

#[must_use]
fn before_each_1(conn: &PgConnection) -> (Account, Namespace) {
    use shared::db::{
        create_account, create_namespace, create_operations, AccountKind, NamespaceKind,
    };

    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    let _foxford_account = create_account(conn, AccountKind::Foxford);

    diesel::insert_into(abac_object::table)
        .values(AbacObject {
            inbound: AbacAttribute {
                namespace_id: iam_namespace.id,
                key: "type".to_owned(),
                value: "namespace".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: iam_namespace.id,
                key: "uri".to_owned(),
                value: format!("namespace/{}", iam_namespace.id),
            },
        })
        .execute(conn)
        .unwrap();

    (iam_account, iam_namespace)
}

mod with_permission {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> (Account, Namespace) {
        let (iam_account, iam_namespace) = before_each_1(conn);

        diesel::insert_into(abac_policy::table)
            .values(AbacPolicy {
                subject: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", iam_account.id),
                }],
                object: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", iam_account.id),
                }],
                action: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                }],
                namespace_id: iam_namespace.id,
            })
            .execute(conn)
            .unwrap();

        (iam_account, iam_namespace)
    }

    #[test]
    fn when_authorized_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();

        if let Ok(resp) = serde_json::from_slice::<jsonrpc::Success>(&body) {
            let namespace: Namespace = serde_json::from_value(resp.result).unwrap();

            let expected = build_record();
            assert_ne!(namespace.id, Uuid::nil());
            assert_eq!(namespace.label, expected.label);
            assert_eq!(namespace.account_id, expected.account_id);
            assert_eq!(namespace.enabled, expected.enabled);

            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn), Ok(1));
        } else {
            panic!("{:?}", body);
        }
    }

    #[test]
    fn when_anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        let conn = get_conn!(pool);
        assert_eq!(find_record(&conn), Ok(0));
    }
}

mod without_permission {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> (Account, Namespace) {
        before_each_1(conn)
    }

    #[test]
    fn when_authorized_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        let conn = get_conn!(pool);
        assert_eq!(find_record(&conn), Ok(0));
    }

    #[test]
    fn when_anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        let conn = get_conn!(pool);
        assert_eq!(find_record(&conn), Ok(0));
    }
}

fn build_request() -> serde_json::Value {
    let namespace = build_record();
    json!({
        "jsonrpc": "2.0",
        "method": "namespace.create",
        "params": [namespace],
        "id": "qwerty"
    })
}

fn build_record() -> NewNamespace {
    NewNamespace {
        label: "foxford.ru".to_owned(),
        account_id: *FOXFORD_ACCOUNT_ID,
        enabled: true,
    }
}

fn find_record(conn: &PgConnection) -> diesel::QueryResult<usize> {
    let namespace = build_record();
    namespace::table
        .filter(namespace::label.eq(namespace.label))
        .filter(namespace::account_id.eq(namespace.account_id))
        .filter(namespace::enabled.eq(namespace.enabled))
        .execute(conn)
}
