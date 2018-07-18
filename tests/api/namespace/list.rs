use actix_web::HttpMessage;
use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use iam::models::{Account, Namespace};
use iam::schema::namespace;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, NETOLOGY_ACCOUNT_ID};

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
    let netology_namespace = create_namespace(conn, NamespaceKind::Netology(netology_account.id));

    diesel::update(&netology_namespace)
        .set(namespace::deleted_at.eq(diesel::dsl::now))
        .execute(conn)
        .unwrap();

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

#[test]
fn admin_can_list_any_client_namespaces() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
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
                "data": {
                    "account_id": "FOXFORD_ACCOUNT_ID",
                    "created_at": "2018-05-30T08:40:00Z",
                    "label": "foxford.ru"
                },
                "id": "FOXFORD_NAMESPACE_ID"
            }
        ],
        "id": "qwerty"
    }"#;
    let resp_json = resp_template
        .replace("FOXFORD_ACCOUNT_ID", &FOXFORD_ACCOUNT_ID.to_string())
        .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());
    assert_eq!(body, shared::strip_json(&resp_json));
}

mod with_client {
    use super::*;

    #[test]
    fn can_list_own_namespaces() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
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
                    "data": {
                        "account_id": "FOXFORD_ACCOUNT_ID",
                        "created_at": "2018-05-30T08:40:00Z",
                        "label": "foxford.ru"
                    },
                    "id": "FOXFORD_NAMESPACE_ID"
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
    fn cannot_list_alien_namespaces() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(*NETOLOGY_ACCOUNT_ID)).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn cannot_list_admin_namespaces() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
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
fn anonymous_cannot_list_namespaces() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let req = shared::build_anonymous_request(
        &srv,
        serde_json::to_string(&build_request(*FOXFORD_ACCOUNT_ID)).unwrap(),
    );
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::FORBIDDEN);
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
