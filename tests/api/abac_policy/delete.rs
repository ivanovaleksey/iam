use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::models::AbacPolicy;
use abac::schema::abac_policy;
use abac::AbacAttribute;

use iam::abac_attribute::{CollectionKind, OperationKind, UriKind};
use iam::models::{Account, Namespace};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_NAMESPACE_ID, NETOLOGY_ACCOUNT_ID,
};

lazy_static! {
    static ref ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "action": [
                    {
                        "key": "operation",
                        "namespace_id": "IAM_NAMESPACE_ID",
                        "value": "any"
                    }
                ],
                "namespace_id": "FOXFORD_NAMESPACE_ID",
                "object": [
                    {
                        "key": "uri",
                        "namespace_id": "IAM_NAMESPACE_ID",
                        "value": "namespace/FOXFORD_NAMESPACE_ID"
                    }
                ],
                "subject": [
                    {
                        "key": "uri",
                        "namespace_id": "IAM_NAMESPACE_ID",
                        "value": "account/ACCOUNT_ID"
                    }
                ]
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("ACCOUNT_ID", &ACCOUNT_ID.to_string());

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

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> AbacPolicy {
        let _ = before_each_1(conn);
        create_record(conn)
    }

    mod with_client {
        use super::*;

        #[test]
        fn can_delete_own_record() {
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
            assert_eq!(body, *EXPECTED);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn), Ok(0));
            }
        }

        #[test]
        fn cannot_delete_alien_record() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
                let _ = create_account(&conn, AccountKind::Netology);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*NETOLOGY_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn), Ok(1));
            }
        }

        #[test]
        fn can_delete_alien_record_when_permission_granted() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);

                let netology_account = create_account(&conn, AccountKind::Netology);

                diesel::insert_into(abac_policy::table)
                    .values(AbacPolicy {
                        subject: vec![AbacAttribute::new(
                            *IAM_NAMESPACE_ID,
                            UriKind::Account(netology_account.id),
                        )],
                        object: vec![
                            AbacAttribute::new(
                                *IAM_NAMESPACE_ID,
                                UriKind::Namespace(*FOXFORD_NAMESPACE_ID),
                            ),
                            AbacAttribute::new(*IAM_NAMESPACE_ID, CollectionKind::AbacPolicy),
                        ],
                        action: vec![AbacAttribute::new(*IAM_NAMESPACE_ID, OperationKind::Delete)],
                        namespace_id: *IAM_NAMESPACE_ID,
                    })
                    .execute(&conn)
                    .unwrap();
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*NETOLOGY_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *EXPECTED);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn), Ok(0));
            }
        }
    }

    #[test]
    fn anonymous_cannot_delete_record() {
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

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn), Ok(1));
        }
    }
}

mod without_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) {
        let _ = before_each_1(conn);
    }

    mod with_client {
        use super::*;

        #[test]
        fn can_delete_own_record() {
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
        fn cannot_delete_alien_record() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*NETOLOGY_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);
        }
    }

    #[test]
    fn anonymous_cannot_delete_record() {
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
    }
}

fn build_request() -> serde_json::Value {
    let policy = build_record();
    json!({
        "jsonrpc": "2.0",
        "method": "abac_policy.delete",
        "params": [policy],
        "id": "qwerty"
    })
}

fn build_record() -> AbacPolicy {
    AbacPolicy {
        namespace_id: *FOXFORD_NAMESPACE_ID,
        subject: vec![AbacAttribute {
            namespace_id: *IAM_NAMESPACE_ID,
            key: "uri".to_owned(),
            value: format!("account/{}", *ACCOUNT_ID),
        }],
        object: vec![AbacAttribute {
            namespace_id: *IAM_NAMESPACE_ID,
            key: "uri".to_owned(),
            value: format!("namespace/{}", *FOXFORD_NAMESPACE_ID),
        }],
        action: vec![AbacAttribute {
            namespace_id: *IAM_NAMESPACE_ID,
            key: "operation".to_owned(),
            value: "any".to_owned(),
        }],
    }
}

fn create_record(conn: &PgConnection) -> AbacPolicy {
    diesel::insert_into(abac_policy::table)
        .values(build_record())
        .get_result(conn)
        .unwrap()
}

fn find_record(conn: &PgConnection) -> diesel::QueryResult<usize> {
    let policy = build_record();
    abac_policy::table
        .find((
            policy.subject,
            policy.object,
            policy.action,
            policy.namespace_id,
        ))
        .execute(conn)
}
