use actix_web::HttpMessage;
use diesel::{self, prelude::*};
use jsonrpc;
use serde_json;
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

        let _namespace = diesel::insert_into(namespace::table)
            .values((
                namespace::id.eq(namespace_id),
                namespace::label.eq("example.org"),
                namespace::account_id.eq(account.id),
                namespace::enabled.eq(true),
            ))
            .get_result::<Namespace>(&conn)
            .unwrap();
    }

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "abac_policy.create",
        "params": [{
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "subject_key": "role",
            "subject_value": "client",
            "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "object_key": "type",
            "object_value": "identity",
            "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "action_key": "action",
            "action_value": "*"
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    if let Ok(resp) = serde_json::from_slice::<jsonrpc::Success>(&body) {
        let policy: AbacPolicy = serde_json::from_value(resp.result).unwrap();

        assert_eq!(policy.namespace_id, namespace_id);
        assert_eq!(policy.subject_namespace_id, namespace_id);
        assert_eq!(policy.subject_key, "role".to_owned());
        assert_eq!(policy.subject_value, "client".to_owned());
        assert_eq!(policy.object_namespace_id, namespace_id);
        assert_eq!(policy.object_key, "type".to_owned());
        assert_eq!(policy.object_value, "identity".to_owned());
        assert_eq!(policy.action_namespace_id, namespace_id);
        assert_eq!(policy.action_key, "action".to_owned());
        assert_eq!(policy.action_value, "*".to_owned());
    } else {
        panic!("{:?}", body);
    }
}
