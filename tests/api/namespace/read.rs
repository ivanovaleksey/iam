use diesel::{self, prelude::*};
use serde_json;

use abac::models::{AbacObject, AbacPolicy};
use abac::schema::{abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::models::{Account, Namespace};
use iam::schema::namespace;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID};

lazy_static! {
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "account_id": "FOXFORD_ACCOUNT_ID",
                "created_at": "2018-05-30T08:40:00",
                "enabled": true,
                "id": "FOXFORD_NAMESPACE_ID",
                "label": "foxford.ru"
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("FOXFORD_ACCOUNT_ID", &FOXFORD_ACCOUNT_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

        shared::strip_json(&json)
    };
}

#[must_use]
fn before_each_1(conn: &PgConnection) -> (Account, Namespace) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

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

    mod with_enabled_namespace {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_3(conn: &PgConnection) -> Namespace {
            let _ = before_each_2(conn);

            let foxford_account = create_account(conn, AccountKind::Foxford);
            let foxford_namespace =
                create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

            foxford_namespace
        }

        #[test]
        fn when_authorized_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *EXPECTED);
        }

        #[test]
        fn when_anonymous_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_anonymous_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    mod with_disabled_namespace {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_3(conn: &PgConnection) -> Namespace {
            let _ = before_each_2(conn);

            let foxford_account = create_account(conn, AccountKind::Foxford);
            let foxford_namespace =
                create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

            diesel::update(&foxford_namespace)
                .set(namespace::enabled.eq(false))
                .execute(conn)
                .unwrap();

            foxford_namespace
        }

        #[test]
        fn when_authorized_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::NOT_FOUND);
        }

        #[test]
        fn when_anonymous_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_anonymous_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    mod without_existing_record {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_3(conn: &PgConnection) {
            let _ = before_each_2(conn);
        }

        #[test]
        fn when_authorized_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::NOT_FOUND);
        }

        #[test]
        fn when_anonymous_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_anonymous_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }
}

mod without_permission {
    use super::*;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> (Account, Namespace) {
        before_each_1(conn)
    }

    mod with_enabled_namespace {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_3(conn: &PgConnection) -> Namespace {
            let _ = before_each_2(conn);

            let foxford_account = create_account(conn, AccountKind::Foxford);
            let foxford_namespace =
                create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

            foxford_namespace
        }

        #[test]
        fn when_authorized_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn when_anonymous_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_anonymous_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    mod with_disabled_namespace {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_3(conn: &PgConnection) -> Namespace {
            let _ = before_each_2(conn);

            let foxford_account = create_account(conn, AccountKind::Foxford);
            let foxford_namespace =
                create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

            diesel::update(&foxford_namespace)
                .set(namespace::enabled.eq(false))
                .execute(conn)
                .unwrap();

            foxford_namespace
        }

        #[test]
        fn when_authorized_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn when_anonymous_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_anonymous_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    mod without_existing_record {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_3(conn: &PgConnection) {
            let _ = before_each_2(conn);
        }

        #[test]
        fn when_authorized_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn when_anonymous_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_anonymous_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }
}

fn build_request() -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "namespace.read",
        "params": [{
            "id": *FOXFORD_NAMESPACE_ID
        }],
        "id": "qwerty"
    })
}
