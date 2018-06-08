use actix_web::HttpMessage;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

use iam::schema::*;

use shared;

lazy_static! {
    static ref USER_ACCOUNT_ID: Uuid = Uuid::new_v4();
}

fn before_each(conn: &PgConnection) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");
}

#[test]
fn with_existing_record() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        before_each(&conn);

        diesel::insert_into(account::table)
            .values(account::id.eq(*USER_ACCOUNT_ID))
            .execute(&conn)
            .unwrap();
    }

    let req_json = json!({
        "jsonrpc": "2.0",
        "method": "account.read",
        "params": [{
            "id": *USER_ACCOUNT_ID,
        }],
        "id": "qwerty"
    });
    let req = shared::build_auth_request(&srv, serde_json::to_string(&req_json).unwrap(), None);

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp_template = r#"{
        "jsonrpc": "2.0",
        "result": {
            "id": "ACCOUNT_ID"
        },
        "id": "qwerty"
    }"#;
    let resp_json = resp_template.replace("ACCOUNT_ID", &USER_ACCOUNT_ID.to_string());
    assert_eq!(body, shared::strip_json(&resp_json));
}

#[test]
fn with_nonexisting_record() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        before_each(&conn);
    }

    let req_json = json!({
        "jsonrpc": "2.0",
        "method": "account.read",
        "params": [{
            "id": *USER_ACCOUNT_ID,
        }],
        "id": "qwerty"
    });
    let req = shared::build_auth_request(&srv, serde_json::to_string(&req_json).unwrap(), None);

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
    assert_eq!(body, shared::strip_json(resp_json));
}
