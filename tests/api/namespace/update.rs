use actix_web::HttpMessage;
use diesel;
use diesel::prelude::*;

use iam::schema::*;

use shared;

#[test]
fn change_label() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(&conn);
        let _ = shared::db::create_iam_namespace(&conn, account.id);
    }

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.update",
        "params": [{
            "id": "bab37008-3dc5-492c-af73-80c241241d71",
            "label": "chat.ng.services"
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "result": {
            "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "created_at": "2018-05-30T08:40:00",
            "enabled": true,
            "id": "bab37008-3dc5-492c-af73-80c241241d71",
            "label": "chat.ng.services"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));
}

#[test]
fn disable_record() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(&conn);
        let _ = shared::db::create_iam_namespace(&conn, account.id);
    }

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.update",
        "params": [{
            "id": "bab37008-3dc5-492c-af73-80c241241d71",
            "enabled": false
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "result": {
            "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "created_at": "2018-05-30T08:40:00",
            "enabled": false,
            "id": "bab37008-3dc5-492c-af73-80c241241d71",
            "label": "iam.ng.services"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.read",
        "params": [{
            "id": "bab37008-3dc5-492c-af73-80c241241d71"
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "error": {
            "code": 404,
            "message": "NotFound"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));
}

#[test]
fn enable_record() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(&conn);
        let namespace = shared::db::create_iam_namespace(&conn, account.id);

        diesel::update(&namespace)
            .set(namespace::enabled.eq(false))
            .execute(&conn)
            .unwrap();
    }

    let read_req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.read",
        "params": [{
            "id": "bab37008-3dc5-492c-af73-80c241241d71"
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, read_req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "error": {
            "code": 404,
            "message": "NotFound"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.update",
        "params": [{
            "id": "bab37008-3dc5-492c-af73-80c241241d71",
            "enabled": true
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "result": {
            "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "created_at": "2018-05-30T08:40:00",
            "enabled": true,
            "id": "bab37008-3dc5-492c-af73-80c241241d71",
            "label": "iam.ng.services"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));

    let req = shared::build_anonymous_request(&srv, read_req_json.to_owned());
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "result": {
            "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "created_at": "2018-05-30T08:40:00",
            "enabled": true,
            "id": "bab37008-3dc5-492c-af73-80c241241d71",
            "label": "iam.ng.services"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));
}

#[test]
fn with_nonexisting_record() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let _ = shared::db::create_iam_account(&conn);
    }

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.update",
        "params": [{
            "id": "bab37008-3dc5-492c-af73-80c241241d71",
            "label": "chat.ng.services"
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "error": {
            "code": 404,
            "message": "NotFound"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));
}
