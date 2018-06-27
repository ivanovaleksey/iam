use chrono::NaiveDate;
use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::models::{AbacObject, AbacPolicy};
use abac::schema::{abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::models::{Account, Identity, Namespace};
use iam::schema::{account, identity};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, IAM_NAMESPACE_ID};

lazy_static! {
    static ref FOXFORD_USER_1_ID: Uuid = Uuid::new_v4();
    static ref FOXFORD_USER_2_ID: Uuid = Uuid::new_v4();
    static ref NETOLOGY_NAMESPACE_ID: Uuid = Uuid::new_v4();
    static ref NETOLOGY_USER_ID: Uuid = Uuid::new_v4();
    static ref USER_1_ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref USER_2_ACCOUNT_ID: Uuid = Uuid::new_v4();
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

    let netology_account = create_account(conn, AccountKind::Other(Uuid::new_v4()));
    let _netology_namespace = create_namespace(
        conn,
        NamespaceKind::Other {
            id: *NETOLOGY_NAMESPACE_ID,
            label: "netology.ru",
            account_id: netology_account.id,
        },
    );

    create_records(conn);

    diesel::insert_into(abac_object::table)
        .values(AbacObject {
            inbound: AbacAttribute {
                namespace_id: iam_namespace.id,
                key: "type".to_owned(),
                value: "identity".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: iam_namespace.id,
                key: "uri".to_owned(),
                value: format!("namespace/{}", iam_namespace.id),
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
    use actix_web::HttpMessage;

    fn before_each_2(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
        let ((iam_account, iam_namespace), (foxford_account, foxford_namespace)) =
            before_each_1(conn);

        diesel::insert_into(abac_policy::table)
            .values(vec![
                AbacPolicy {
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
                },
                AbacPolicy {
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
                },
                AbacPolicy {
                    subject: vec![AbacAttribute {
                        namespace_id: iam_namespace.id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", *USER_1_ACCOUNT_ID),
                    }],
                    object: vec![AbacAttribute {
                        namespace_id: iam_namespace.id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", *USER_1_ACCOUNT_ID),
                    }],
                    action: vec![AbacAttribute {
                        namespace_id: iam_namespace.id,
                        key: "operation".to_owned(),
                        value: "any".to_owned(),
                    }],
                    namespace_id: iam_namespace.id,
                },
                AbacPolicy {
                    subject: vec![AbacAttribute {
                        namespace_id: iam_namespace.id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", *USER_2_ACCOUNT_ID),
                    }],
                    object: vec![AbacAttribute {
                        namespace_id: iam_namespace.id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", *USER_2_ACCOUNT_ID),
                    }],
                    action: vec![AbacAttribute {
                        namespace_id: iam_namespace.id,
                        key: "operation".to_owned(),
                        value: "any".to_owned(),
                    }],
                    namespace_id: iam_namespace.id,
                },
            ])
            .execute(conn)
            .unwrap();

        (
            (iam_account, iam_namespace),
            (foxford_account, foxford_namespace),
        )
    }

    mod with_authorized_admin {
        use super::*;

        #[test]
        fn without_filter() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:01",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_1_ID"
                    },
                    {
                        "account_id": "USER_2_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:02",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_2_ID"
                    },
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:03",
                        "label": "oauth2",
                        "provider": "NETOLOGY_NAMESPACE_ID",
                        "uid": "NETOLOGY_USER_ID"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
                .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
                .replace("NETOLOGY_NAMESPACE_ID", &NETOLOGY_NAMESPACE_ID.to_string())
                .replace("NETOLOGY_USER_ID", &NETOLOGY_USER_ID.to_string())
                .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string())
                .replace("USER_2_ACCOUNT_ID", &USER_2_ACCOUNT_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn with_filter_by_provider() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:01",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_1_ID"
                    },
                    {
                        "account_id": "USER_2_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:02",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_2_ID"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
                .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
                .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string())
                .replace("USER_2_ACCOUNT_ID", &USER_2_ACCOUNT_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn with_filter_by_provider_and_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:01",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_1_ID"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
                .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn with_filter_by_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, Some(*USER_2_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "USER_2_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:02",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_2_ID"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
                .replace("USER_2_ACCOUNT_ID", &USER_2_ACCOUNT_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }
    }

    mod with_authorized_client {
        use super::*;

        #[test]
        fn without_filter() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_own_provider() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:01",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_1_ID"
                    },
                    {
                        "account_id": "USER_2_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:02",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_2_ID"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
                .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
                .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string())
                .replace("USER_2_ACCOUNT_ID", &USER_2_ACCOUNT_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn with_filter_by_alien_provider() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*NETOLOGY_NAMESPACE_ID), None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_provider_and_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:01",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_1_ID"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
                .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn with_filter_by_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    mod with_authorized_user {
        use super::*;

        #[test]
        fn without_filter() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_provider() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_provider_and_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:01",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_1_ID"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
                .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn with_filter_by_own_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:01",
                        "label": "oauth2",
                        "provider": "FOXFORD_NAMESPACE_ID",
                        "uid": "FOXFORD_USER_1_ID"
                    },
                    {
                        "account_id": "USER_1_ACCOUNT_ID",
                        "created_at": "2018-06-02T08:40:03",
                        "label": "oauth2",
                        "provider": "NETOLOGY_NAMESPACE_ID",
                        "uid": "NETOLOGY_USER_ID"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("NETOLOGY_NAMESPACE_ID", &NETOLOGY_NAMESPACE_ID.to_string())
                .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
                .replace("NETOLOGY_USER_ID", &NETOLOGY_USER_ID.to_string())
                .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn with_filter_by_alien_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, Some(*USER_2_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    #[test]
    fn when_anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
        let req = shared::build_anonymous_request(&srv, serde_json::to_string(&payload).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

mod without_permission {
    use super::*;
    use actix_web::HttpMessage;

    fn before_each_2(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
        before_each_1(conn)
    }

    mod with_authorized_admin {
        use super::*;

        #[test]
        fn without_filter() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_provider() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_provider_and_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, Some(*USER_2_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    mod with_authorized_client {
        use super::*;

        #[test]
        fn without_filter() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_own_provider() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_alien_provider() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*NETOLOGY_NAMESPACE_ID), None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_provider_and_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    mod with_authorized_user {
        use super::*;

        #[test]
        fn without_filter() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_provider() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_provider_and_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_own_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, Some(*USER_1_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }

        #[test]
        fn with_filter_by_alien_account() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let payload = build_request(None, Some(*USER_2_ACCOUNT_ID));
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*USER_1_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    #[test]
    fn when_anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
        let req = shared::build_anonymous_request(&srv, serde_json::to_string(&payload).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

fn build_request(provider: Option<Uuid>, account_id: Option<Uuid>) -> serde_json::Value {
    let mut filter = json!({});

    if let Some(provider) = provider {
        filter["provider"] = serde_json::to_value(provider).unwrap();
    }
    if let Some(account_id) = account_id {
        filter["account_id"] = serde_json::to_value(account_id).unwrap();
    }

    json!({
        "jsonrpc": "2.0",
        "method": "identity.list",
        "params": [{
            "filter": filter
        }],
        "id": "qwerty"
    })
}

fn create_records(conn: &PgConnection) {
    let user_1_account = diesel::insert_into(account::table)
        .values((
            account::id.eq(*USER_1_ACCOUNT_ID),
            account::enabled.eq(true),
        ))
        .get_result::<Account>(conn)
        .unwrap();

    let user_2_account = diesel::insert_into(account::table)
        .values((
            account::id.eq(*USER_2_ACCOUNT_ID),
            account::enabled.eq(true),
        ))
        .get_result::<Account>(conn)
        .unwrap();

    let identities = vec![
        (
            identity::provider.eq(*FOXFORD_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_1_ID.to_string()),
            identity::account_id.eq(user_1_account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 1)),
        ),
        (
            identity::provider.eq(*FOXFORD_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_2_ID.to_string()),
            identity::account_id.eq(user_2_account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 2)),
        ),
        (
            identity::provider.eq(*NETOLOGY_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(NETOLOGY_USER_ID.to_string()),
            identity::account_id.eq(user_1_account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 3)),
        ),
    ];

    for identity in &identities {
        let identity = diesel::insert_into(identity::table)
            .values(identity)
            .get_result::<Identity>(conn)
            .unwrap();

        use iam::models::identity::PrimaryKey;
        let pk = PrimaryKey::from(identity.clone());

        diesel::insert_into(abac_object::table)
            .values(vec![
                AbacObject {
                    inbound: AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("identity/{}", pk),
                    },
                    outbound: AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "type".to_owned(),
                        value: "identity".to_owned(),
                    },
                },
                AbacObject {
                    inbound: AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("identity/{}", pk),
                    },
                    outbound: AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("namespace/{}", identity.provider),
                    },
                },
                AbacObject {
                    inbound: AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("identity/{}", pk),
                    },
                    outbound: AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("account/{}", identity.account_id),
                    },
                },
            ])
            .execute(conn)
            .unwrap();
    }
}
