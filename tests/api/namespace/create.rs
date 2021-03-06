use actix_web::HttpMessage;
use diesel::{self, prelude::*};
use jsonrpc;
use serde_json;
use uuid::Uuid;

use iam::models::{Account, Namespace, NewNamespace};
use iam::schema::namespace;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, IAM_ACCOUNT_ID, IAM_NAMESPACE_ID};

#[must_use]
fn before_each_1(conn: &PgConnection) -> (Account, Namespace) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    let _foxford_account = create_account(conn, AccountKind::Foxford);

    (iam_account, iam_namespace)
}

#[test]
fn admin_can_create_namespace() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let req = shared::build_auth_request(
        &srv,
        serde_json::to_string(&build_request()).unwrap(),
        Some(*IAM_ACCOUNT_ID),
    );
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();

    if let Ok(resp) = serde_json::from_slice::<jsonrpc::Success>(&body) {
        use iam::rpc::namespace::create::Response;

        let resp: Response = serde_json::from_value(resp.result).unwrap();
        let namespace_id = resp.id;

        let expected = build_record();
        assert_ne!(namespace_id, Uuid::nil());
        assert_eq!(resp.data.label, expected.label);
        assert_eq!(resp.data.account_id, expected.account_id);

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn), Ok(1));
        }

        let req_json = json!({
            "jsonrpc": "2.0",
            "method": "authorize",
            "params": [{
                "namespace_ids": [*IAM_NAMESPACE_ID],
                "subject": [
                    {
                        "namespace_id": *IAM_NAMESPACE_ID,
                        "key": "uri",
                        "value": format!("account/{}", *IAM_ACCOUNT_ID),
                    }
                ],
                "object": [
                    {
                        "namespace_id": *IAM_NAMESPACE_ID,
                        "key": "uri",
                        "value": format!("namespace/{}", namespace_id),
                    }
                ],
                "action": [
                    {
                        "namespace_id": *IAM_NAMESPACE_ID,
                        "key": "operation",
                        "value": "read",
                    },
                    {
                        "namespace_id": *IAM_NAMESPACE_ID,
                        "key": "operation",
                        "value": "update",
                    },
                    {
                        "namespace_id": *IAM_NAMESPACE_ID,
                        "key": "operation",
                        "value": "delete",
                    }
                ],
            }],
            "id": "qwerty",
        });
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&req_json).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "result": true,
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(resp_json));
    } else {
        panic!("{:?}", body);
    }
}

#[test]
fn client_cannot_create_namespace() {
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

    let conn = get_conn!(pool);
    assert_eq!(find_record(&conn), Ok(0));
}

#[test]
fn anonymous_cannot_create_namespace() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let req =
        shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::FORBIDDEN);

    let conn = get_conn!(pool);
    assert_eq!(find_record(&conn), Ok(0));
}

fn build_request() -> serde_json::Value {
    let namespace = build_record();
    json!({
        "jsonrpc": "2.0",
        "method": "namespace.create",
        "params": [{
            "data": {
                "account_id": namespace.account_id,
                "label": namespace.label,
            }
        }],
        "id": "qwerty"
    })
}

fn build_record() -> NewNamespace {
    NewNamespace {
        label: "foxford.ru".to_owned(),
        account_id: *FOXFORD_ACCOUNT_ID,
    }
}

fn find_record(conn: &PgConnection) -> diesel::QueryResult<usize> {
    let namespace = build_record();
    namespace::table
        .filter(namespace::label.eq(namespace.label))
        .filter(namespace::account_id.eq(namespace.account_id))
        .filter(namespace::deleted_at.is_null())
        .execute(conn)
}
