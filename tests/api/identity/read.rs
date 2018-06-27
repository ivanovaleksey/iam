use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

use abac::models::{AbacObject, AbacPolicy};
use abac::schema::{abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::models::{Account, Identity, Namespace};
use iam::schema::{account, identity};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_NAMESPACE_ID};

lazy_static! {
    static ref FOXFORD_USER_ID: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "account_id": "USER_ACCOUNT_ID",
                "created_at": "2018-06-02T08:40:00",
                "label": "oauth2",
                "provider": "FOXFORD_NAMESPACE_ID",
                "uid": "FOXFORD_USER_ID"
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("FOXFORD_USER_ID", &FOXFORD_USER_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("USER_ACCOUNT_ID", &USER_ACCOUNT_ID.to_string());

        shared::strip_json(&json)
    };
}

#[must_use]
fn before_each_1(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
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
                value: "identity".to_owned(),
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

mod with_permission {
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

        diesel::insert_into(abac_policy::table)
            .values(AbacPolicy {
                subject: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", *USER_ACCOUNT_ID),
                }],
                object: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", *USER_ACCOUNT_ID),
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
        fn before_each_3(conn: &PgConnection) -> Identity {
            let _ = before_each_2(conn);
            create_user_identity(conn)
        }

        #[test]
        fn when_authorized_client() {
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
        fn when_authorized_user() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*USER_ACCOUNT_ID),
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

        #[must_use]
        fn before_each_3(conn: &PgConnection) {
            let _ = before_each_2(conn);
        }

        #[test]
        fn when_authorized_client() {
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
            assert_eq!(body, *shared::api::NOT_FOUND);
        }

        #[test]
        fn when_authorized_user() {
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
    fn before_each_2(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
        before_each_1(conn)
    }

    mod with_existing_record {
        use super::*;
        use actix_web::HttpMessage;

        #[must_use]
        fn before_each_3(conn: &PgConnection) -> Identity {
            let _ = before_each_2(conn);
            create_user_identity(conn)
        }

        #[test]
        fn when_authorized_client() {
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
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn when_authorized_user() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*USER_ACCOUNT_ID),
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
        fn when_authorized_client() {
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
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn when_authorized_user() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_3(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*USER_ACCOUNT_ID),
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
        "method": "identity.read",
        "params": [{
            "provider": *FOXFORD_NAMESPACE_ID,
            "label": "oauth2",
            "uid": *FOXFORD_USER_ID,
        }],
        "id": "qwerty"
    })
}

fn create_user_identity(conn: &PgConnection) -> Identity {
    let account = diesel::insert_into(account::table)
        .values((account::id.eq(*USER_ACCOUNT_ID), account::enabled.eq(true)))
        .get_result::<Account>(conn)
        .unwrap();

    let identity = diesel::insert_into(identity::table)
        .values((
            identity::provider.eq(*FOXFORD_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_ID.to_string()),
            identity::account_id.eq(account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
        ))
        .get_result::<Identity>(conn)
        .unwrap();

    use iam::models::identity::PrimaryKey;
    let pk = PrimaryKey::from(identity.clone());

    diesel::insert_into(abac_object::table)
        .values(vec![
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("identity/{}", pk),
                },
                outbound: AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("account/{}", account.id),
                },
            },
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("identity/{}", pk),
                },
                outbound: AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", *FOXFORD_NAMESPACE_ID),
                },
            },
        ])
        .execute(conn)
        .unwrap();

    identity
}
