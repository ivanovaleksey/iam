use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

use abac::models::prelude::*;
use abac::schema::*;
use abac::types::AbacAttribute;
use iam::models::{Account, Namespace};

use shared;

#[must_use]
fn before_each(conn: &PgConnection) -> (Account, Namespace) {
    use shared::db::{create_iam_account, create_iam_namespace};

    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let account = create_iam_account(&conn);
    let namespace = create_iam_namespace(conn, account.id);

    diesel::insert_into(abac_subject::table)
        .values(AbacSubject {
            inbound: AbacAttribute {
                namespace_id: namespace.id,
                key: "uri".to_owned(),
                value: "account/25a0c367-756a-42e1-ac5a-e7a2b6b64420".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: namespace.id,
                key: "role".to_owned(),
                value: "client".to_owned(),
            },
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_object::table)
        .values(AbacObject {
            inbound: AbacAttribute {
                namespace_id: namespace.id,
                key: "uri".to_owned(),
                value: "room/1".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: namespace.id,
                key: "type".to_owned(),
                value: "room".to_owned(),
            },
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_action::table)
        .values(vec![
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id: namespace.id,
                    key: "operation".to_owned(),
                    value: "create".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: namespace.id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id: namespace.id,
                    key: "operation".to_owned(),
                    value: "read".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: namespace.id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
        ])
        .execute(conn)
        .unwrap();

    (account, namespace)
}

#[test]
fn with_permission() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        let (_account, namespace) = before_each(&conn);

        diesel::insert_into(abac_policy::table)
            .values(AbacPolicy {
                subject: vec![AbacAttribute {
                    namespace_id: namespace.id,
                    key: "role".to_owned(),
                    value: "client".to_owned(),
                }],
                object: vec![AbacAttribute {
                    namespace_id: namespace.id,
                    key: "type".to_owned(),
                    value: "room".to_owned(),
                }],
                action: vec![AbacAttribute {
                    namespace_id: namespace.id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                }],
                namespace_id: namespace.id,
            })
            .execute(&conn)
            .unwrap();
    }

    let req =
        shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "result": true,
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(resp_json));
}

#[test]
fn without_permission() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        let _ = before_each(&conn);
    }

    let req =
        shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "result": false,
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(resp_json));
}

fn build_request() -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "authorize",
        "params": [{
            "namespace_ids": ["bab37008-3dc5-492c-af73-80c241241d71"],
            "subject": [
                {
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "key": "uri",
                    "value": "account/25a0c367-756a-42e1-ac5a-e7a2b6b64420"
                }
            ],
            "object": [
                {
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "key": "uri",
                    "value": "room/1"
                }
            ],
            "action": [
                {
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "key": "operation",
                    "value": "read"
                }
            ],
        }],
        "id": "qwerty",
    })
}
