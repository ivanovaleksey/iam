use actix_web::HttpMessage;
use diesel::prelude::*;
use jsonrpc;
use serde_json;
use uuid::Uuid;

use iam::models::*;

use shared;

#[test]
fn test() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let _ = shared::db::create_iam_account(&conn);
    }

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.create",
        "params": [{
            "label": "iam.ng.services",
            "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "enabled": true
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let rpc_resp: jsonrpc::Success = serde_json::from_slice(&body).unwrap();
    let namespace: Namespace = serde_json::from_value(rpc_resp.result).unwrap();

    assert_ne!(namespace.id, Uuid::nil());
    assert_eq!(namespace.label, "iam.ng.services");
    assert_eq!(
        namespace.account_id,
        Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap()
    );
    assert_eq!(namespace.enabled, true);
}
