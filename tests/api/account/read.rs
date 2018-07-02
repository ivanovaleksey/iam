use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::models::{AbacObject, AbacPolicy};
use abac::schema::{abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::models::{Account, Namespace};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, IAM_ACCOUNT_ID, IAM_NAMESPACE_ID};

lazy_static! {
    static ref NETOLOGY_ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref NETOLOGY_NAMESPACE_ID: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID_1: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID_2: Uuid = Uuid::new_v4();
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "id": "USER_ACCOUNT_ID_1"
            },
            "id": "qwerty"
        }"#;

        let json = template.replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string());

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

    let netology_account = create_account(conn, AccountKind::Other(*NETOLOGY_ACCOUNT_ID));
    let _netology_namespace = create_namespace(
        conn,
        NamespaceKind::Other {
            id: *NETOLOGY_NAMESPACE_ID,
            label: "netology.ru",
            account_id: netology_account.id,
        },
    );

    let user_account_2 = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_2));

    diesel::insert_into(abac_object::table)
        .values(AbacObject {
            inbound: AbacAttribute {
                namespace_id: iam_namespace.id,
                key: "type".to_owned(),
                value: "account".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: iam_namespace.id,
                key: "uri".to_owned(),
                value: format!("namespace/{}", iam_namespace.id),
            },
        })
        .execute(conn)
        .unwrap();

    [&iam_account, &foxford_account, &user_account_2]
        .iter()
        .for_each(|account| {
            diesel::insert_into(abac_policy::table)
                .values(AbacPolicy {
                    subject: vec![AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("account/{}", account.id),
                    }],
                    object: vec![AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("account/{}", account.id),
                    }],
                    action: vec![AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "operation".to_owned(),
                        value: "any".to_owned(),
                    }],
                    namespace_id: *IAM_NAMESPACE_ID,
                })
                .execute(conn)
                .unwrap();
        });

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> Account {
        let _ = before_each_1(conn);

        let account = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_1));

        diesel::insert_into(abac_policy::table)
            .values(AbacPolicy {
                subject: vec![AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("account/{}", account.id),
                }],
                object: vec![AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("account/{}", account.id),
                }],
                action: vec![AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                }],
                namespace_id: *IAM_NAMESPACE_ID,
            })
            .execute(conn)
            .unwrap();

        account
    }

    #[test]
    fn admin_can_read_any_account() {
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
        assert_eq!(body, *EXPECTED);
    }

    #[test]
    fn client_cannot_read_user_account() {
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
    fn user_can_read_own_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *EXPECTED);
    }

    #[test]
    fn user_cannot_read_alien_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*USER_ACCOUNT_ID_2),
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

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

mod without_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) {
        let _ = before_each_1(conn);
    }

    #[test]
    fn admin_can_read_any_account() {
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
        assert_eq!(body, *shared::api::NOT_FOUND);
    }

    #[test]
    fn client_cannot_read_user_account() {
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
    fn user_cannot_read_alien_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*USER_ACCOUNT_ID_2),
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

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

fn build_request() -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "account.read",
        "params": [{
            "id": *USER_ACCOUNT_ID_1
        }],
        "id": "qwerty"
    })
}
