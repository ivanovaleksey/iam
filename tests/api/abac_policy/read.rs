use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::models::*;
use iam::schema::*;

use shared;

#[test]
fn with_existing_record() {
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

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("identity"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("action"),
                abac_policy::action_value.eq("*"),
                abac_policy::created_at.eq(NaiveDate::from_ymd(2018, 5, 29).and_hms(7, 15, 0)),
            ))
            .execute(&conn)
            .unwrap();
    }

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "abac_policy.read",
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
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "result": {
            "action_key": "action",
            "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "action_value": "*",
            "created_at":"2018-05-29T07:15:00",
            "expired_at": null,
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "not_before": null,
            "object_key": "type",
            "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "object_value": "identity",
            "subject_key": "role",
            "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "subject_value": "client"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&resp_json));
}

#[test]
fn with_nonexisting_record() {
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
        "method": "abac_policy.read",
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
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "error": {
            "code": 404,
            "message": "NotFound"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&resp_json));
}
