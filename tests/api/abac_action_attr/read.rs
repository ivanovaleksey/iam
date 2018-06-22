use diesel::{self, prelude::*};
use serde_json;

use abac::models::{AbacAction, AbacObject, AbacPolicy};
use abac::schema::{abac_action, abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::models::{Account, Namespace};

use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_NAMESPACE_ID};

lazy_static! {
    static ref OPERATION: &'static str = "execute";
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "operation",
                    "namespace_id": "IAM_NAMESPACE_ID",
                    "value": "OPERATION"
                },
                "outbound": {
                    "key": "operation",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "any"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("OPERATION", &OPERATION.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

        shared::strip_json(&json)
    };
}

#[must_use]
fn before_each_1(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
    use shared::db::{
        create_account, create_namespace, create_operations, AccountKind, NamespaceKind,
    };

    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    let foxford_account = create_account(conn, AccountKind::Foxford);
    let foxford_namespace = create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

    diesel::insert_into(abac_object::table)
        .values(AbacObject {
            inbound: AbacAttribute {
                namespace_id: foxford_namespace.id,
                key: "type".to_owned(),
                value: "abac_action".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: iam_namespace.id,
                key: "uri".to_owned(),
                value: format!("namespace/{}", foxford_namespace.id),
            },
        })
        .execute(conn)
        .unwrap();

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_namespace_ownership {
    use super::*;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
        let ((iam_account, iam_namespace), (foxford_account, foxford_namespace)) =
            before_each_1(conn);

        diesel::insert_into(abac_policy::table)
            .values(AbacPolicy {
                subject: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", foxford_account.id),
                }],
                object: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", foxford_account.id),
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

        (
            (iam_account, iam_namespace),
            (foxford_account, foxford_namespace),
        )
    }

    mod with_existing_record {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_3(conn: &PgConnection) -> AbacAction {
            let _ = before_each_2(conn);
            create_action(conn)
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
                Some(*FOXFORD_ACCOUNT_ID),
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

    mod without_existing_record {
        use super::*;
        use actix_web::HttpMessage;

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
                Some(*FOXFORD_ACCOUNT_ID),
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
                let _ = before_each_2(&conn);
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

mod without_namespace_ownership {
    use super::*;

    mod with_existing_record {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_2(conn: &PgConnection) -> AbacAction {
            let _ = before_each_1(conn);
            create_action(conn)
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
                Some(*FOXFORD_ACCOUNT_ID),
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
                let _ = before_each_2(&conn);
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

        #[test]
        fn when_authorized_request() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_1(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
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
                let _ = before_each_1(&conn);
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
    let action = build_action();
    json!({
        "jsonrpc": "2.0",
        "method": "abac_action_attr.read",
        "params": [action],
        "id": "qwerty"
    })
}

fn build_action() -> AbacAction {
    AbacAction {
        inbound: AbacAttribute {
            namespace_id: *IAM_NAMESPACE_ID,
            key: "operation".to_owned(),
            value: OPERATION.to_owned(),
        },
        outbound: AbacAttribute {
            namespace_id: *FOXFORD_NAMESPACE_ID,
            key: "operation".to_owned(),
            value: "any".to_owned(),
        },
    }
}

fn create_action(conn: &PgConnection) -> AbacAction {
    diesel::insert_into(abac_action::table)
        .values(build_action())
        .get_result(conn)
        .unwrap()
}
