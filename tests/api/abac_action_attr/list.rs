use actix_web::HttpMessage;
use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::models::*;
use iam::schema::*;

use shared;

#[test]
fn test() {
    let shared::Server { mut srv, pool } = shared::build_server();

    let account_id = Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap();
    let namespace_id = Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = diesel::insert_into(account::table)
            .values((account::id.eq(account_id), account::enabled.eq(true)))
            .get_result::<Account>(&conn)
            .unwrap();

        let namespace = diesel::insert_into(namespace::table)
            .values((
                namespace::id.eq(namespace_id),
                namespace::label.eq("example.org"),
                namespace::account_id.eq(account.id),
                namespace::enabled.eq(true),
            ))
            .get_result::<Namespace>(&conn)
            .unwrap();

        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("create"),
                abac_action_attr::key.eq("access"),
                abac_action_attr::value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("read"),
                abac_action_attr::key.eq("access"),
                abac_action_attr::value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();
    }

    let json = r#"{
        "jsonrpc": "2.0",
        "method": "abac_action_attr.list",
        "params": [{
            "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND key:access"
        }],
        "id": "qwerty"
    }"#;
    let req = srv.get().body(json).unwrap();

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "result": [
            {
                "action_id": "create",
                "key": "access",
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "value": "*"
            },
            {
                "action_id": "read",
                "key": "access",
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "value": "*"
            }
        ],
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));
}