use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::prelude::*;
use abac::schema::*;

use iam::abac_attribute::{CollectionKind, OperationKind, UriKind};
use iam::models::{Account, Namespace};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_NAMESPACE_ID, NETOLOGY_ACCOUNT_ID,
};

lazy_static! {
    static ref USER_ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "uri",
                    "namespace_id": "IAM_NAMESPACE_ID",
                    "value": "account/USER_ACCOUNT_ID"
                },
                "outbound": {
                    "key": "role",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "user"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("USER_ACCOUNT_ID", &USER_ACCOUNT_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

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

    let netology_account = create_account(conn, AccountKind::Netology);
    let _netology_namespace = create_namespace(conn, NamespaceKind::Netology(netology_account.id));

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) {
        let _ = before_each_1(conn);

        diesel::insert_into(abac_subject::table)
            .values(build_record())
            .execute(conn)
            .unwrap();
    }

    mod with_client {
        use super::*;

        mod with_own_user {
            use super::*;

            #[must_use]
            fn before_each_3(conn: &PgConnection) {
                let _ = before_each_2(conn);
                create_user_identity(conn);
            }

            #[test]
            fn can_delete_own_record() {
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
                    let _ = before_each_3(&conn);
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
                    let _ = before_each_3(&conn);
                    grant_permission(&conn);
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

        mod with_alien_user {
            use super::*;

            #[must_use]
            fn before_each_3(conn: &PgConnection) {
                let _ = before_each_2(conn);
            }

            #[test]
            fn can_delete_own_record() {
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

                {
                    let conn = get_conn!(pool);
                    assert_eq!(find_record(&conn), Ok(0));
                }
            }

            #[test]
            fn can_delete_alien_record_when_permission_granted() {
                let shared::Server { mut srv, pool } = shared::build_server();

                {
                    let conn = get_conn!(pool);
                    let _ = before_each_3(&conn);
                    grant_permission(&conn);
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

        fn grant_permission(conn: &PgConnection) {
            diesel::insert_into(abac_policy::table)
                .values(NewAbacPolicy {
                    subject: vec![AbacAttribute::new(
                        *IAM_NAMESPACE_ID,
                        UriKind::Account(*NETOLOGY_ACCOUNT_ID),
                    )],
                    object: vec![
                        AbacAttribute::new(
                            *IAM_NAMESPACE_ID,
                            UriKind::Namespace(*FOXFORD_NAMESPACE_ID),
                        ),
                        AbacAttribute::new(*IAM_NAMESPACE_ID, CollectionKind::AbacSubject),
                    ],
                    action: vec![AbacAttribute::new(*IAM_NAMESPACE_ID, OperationKind::Delete)],
                    namespace_id: *IAM_NAMESPACE_ID,
                })
                .execute(conn)
                .unwrap();
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

        mod with_own_user {
            use super::*;

            #[must_use]
            fn before_each_3(conn: &PgConnection) {
                let _ = before_each_2(conn);
                create_user_identity(conn);
            }

            #[test]
            fn can_delete_own_record() {
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
            fn cannot_delete_alien_record() {
                let shared::Server { mut srv, pool } = shared::build_server();

                {
                    let conn = get_conn!(pool);
                    let _ = before_each_3(&conn);
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

        mod with_alien_user {
            use super::*;

            #[must_use]
            fn before_each_3(conn: &PgConnection) {
                let _ = before_each_2(conn);
            }

            #[test]
            fn can_delete_own_record() {
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
    let record = build_record();

    json!({
        "jsonrpc": "2.0",
        "method": "abac_subject_attr.delete",
        "params": [record],
        "id": "qwerty"
    })
}

fn build_record() -> NewAbacSubject {
    NewAbacSubject {
        inbound: AbacAttribute {
            namespace_id: *IAM_NAMESPACE_ID,
            key: "uri".to_owned(),
            value: format!("account/{}", *USER_ACCOUNT_ID),
        },
        outbound: AbacAttribute {
            namespace_id: *FOXFORD_NAMESPACE_ID,
            key: "role".to_owned(),
            value: "user".to_owned(),
        },
    }
}

fn find_record(conn: &PgConnection) -> diesel::QueryResult<usize> {
    let record = build_record();

    abac_subject::table
        .find((record.inbound, record.outbound))
        .execute(conn)
}

fn create_user_identity(conn: &PgConnection) {
    use iam::models::NewIdentity;
    use iam::schema::identity;

    let user_account = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID));

    diesel::insert_into(identity::table)
        .values(NewIdentity {
            provider: *FOXFORD_NAMESPACE_ID,
            label: "oauth2".to_owned(),
            uid: Uuid::new_v4().to_string(),
            account_id: user_account.id,
        })
        .execute(conn)
        .unwrap();
}
