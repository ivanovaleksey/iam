use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::schema::*;

#[test]
fn with_filter() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(&conn);
        let _ = shared::db::create_iam_namespace(&conn, account.id);

        diesel::insert_into(namespace::table)
            .values((
                namespace::id.eq(Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d72").unwrap()),
                namespace::label.eq("chat.ng.services"),
                namespace::account_id.eq(account.id),
                namespace::enabled.eq(false),
                namespace::created_at.eq(NaiveDate::from_ymd(2018, 5, 30).and_hms(8, 40, 1)),
            ))
            .execute(&conn)
            .unwrap();
    }

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.list",
        "params": [{
            "fq": "account_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420"
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "result": [
            {
                "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
                "created_at": "2018-05-30T08:40:00",
                "enabled": true,
                "id": "bab37008-3dc5-492c-af73-80c241241d71",
                "label": "iam.ng.services"
            }
        ],
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));
}

use shared;

#[test]
fn without_filter() {
    let shared::Server { mut srv, pool: _ } = shared::build_server();

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "namespace.list",
        "params": [{
            "fq": ""
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
            "code": -32602,
            "message": "Invalid params: missing field `account_id`."
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&resp_json));
}
