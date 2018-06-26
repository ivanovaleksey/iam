use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::models::{AbacObject, AbacPolicy};
use abac::schema::{abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::models::{Account, Namespace};
use iam::schema::namespace;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID};

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
    let netology_namespace = create_namespace(
        conn,
        NamespaceKind::Other {
            id: Uuid::new_v4(),
            label: "netology.ru",
            account_id: netology_account.id,
        },
    );

    diesel::update(&netology_namespace)
        .set(namespace::enabled.eq(false))
        .execute(conn)
        .unwrap();

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
            ])
            .execute(conn)
            .unwrap();

        (
            (iam_account, iam_namespace),
            (foxford_account, foxford_namespace),
        )
    }

    mod when_authorized_request {
        use super::*;

        #[test]
        fn with_permitted_filter() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request(*FOXFORD_ACCOUNT_ID)).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "account_id": "FOXFORD_ACCOUNT_ID",
                        "created_at": "2018-05-30T08:40:00",
                        "enabled": true,
                        "id": "FOXFORD_NAMESPACE_ID",
                        "label": "foxford.ru"
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_ACCOUNT_ID", &FOXFORD_ACCOUNT_ID.to_string())
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn with_denied_filter() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request(*IAM_ACCOUNT_ID)).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
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

        let req = shared::build_anonymous_request(
            &srv,
            serde_json::to_string(&build_request(*FOXFORD_ACCOUNT_ID)).unwrap(),
        );
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

    #[test]
    fn when_authorized_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(*FOXFORD_ACCOUNT_ID)).unwrap(),
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
            let _ = before_each_2(&conn);
        }

        let req = shared::build_anonymous_request(
            &srv,
            serde_json::to_string(&build_request(*FOXFORD_ACCOUNT_ID)).unwrap(),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

fn build_request(account_id: Uuid) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "namespace.list",
        "params": [{
            "filter": {
                "account_id": account_id
            }
        }],
        "id": "qwerty"
    })
}
