use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use jsonrpc;
use serde_json;
use uuid::Uuid;

use iam::models::prelude::*;
use iam::schema::*;

use shared;

lazy_static! {
    static ref FOXFORD_NAMESPACE_ID: Uuid = Uuid::new_v4();
    static ref FOXFORD_USER_ID: Uuid = Uuid::new_v4();
}

fn before_each(conn: &PgConnection) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let foxford_account = diesel::insert_into(account::table)
        .values((account::id.eq(Uuid::new_v4()), account::enabled.eq(true)))
        .get_result::<Account>(conn)
        .unwrap();

    let _foxford_namespace = diesel::insert_into(namespace::table)
        .values((
            namespace::id.eq(*FOXFORD_NAMESPACE_ID),
            namespace::label.eq("foxford.ru"),
            namespace::account_id.eq(foxford_account.id),
            namespace::enabled.eq(true),
        ))
        .get_result::<Namespace>(conn)
        .unwrap();
}

#[test]
fn create_identity_first_time() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        before_each(&conn);
    }

    let req_json = json!({
        "jsonrpc": "2.0",
        "method": "identity.create",
        "params": [{
            "provider": *FOXFORD_NAMESPACE_ID,
            "label": "oauth2",
            "uid": *FOXFORD_USER_ID,
        }],
        "id": "qwerty"
    });
    let req = shared::build_auth_request(&srv, serde_json::to_string(&req_json).unwrap(), None);

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp = serde_json::from_slice::<jsonrpc::Success>(&body).unwrap();
    let identity = serde_json::from_value::<Identity>(resp.result).unwrap();

    assert_eq!(identity.provider, *FOXFORD_NAMESPACE_ID);
    assert_eq!(identity.label, "oauth2");
    assert_eq!(identity.uid, FOXFORD_USER_ID.to_string());

    let conn = pool.get().expect("Failed to get connection from pool");
    let created_account = account::table
        .find(identity.account_id)
        .get_result::<Account>(&conn)
        .unwrap();

    assert!(created_account.enabled);
}

#[test]
fn create_identity_second_time() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        before_each(&conn);

        let user_account = diesel::insert_into(account::table)
            .values((account::id.eq(Uuid::new_v4()), account::enabled.eq(true)))
            .get_result::<Account>(&conn)
            .unwrap();

        diesel::insert_into(identity::table)
            .values((
                identity::provider.eq(*FOXFORD_NAMESPACE_ID),
                identity::label.eq("oauth2"),
                identity::uid.eq(FOXFORD_USER_ID.to_string()),
                identity::account_id.eq(user_account.id),
                identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
            ))
            .execute(&conn)
            .unwrap();
    }

    let req_json = json!({
        "jsonrpc": "2.0",
        "method": "identity.create",
        "params": [{
            "provider": *FOXFORD_NAMESPACE_ID,
            "label": "oauth2",
            "uid": *FOXFORD_USER_ID,
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
            "code": 422,
            "message": "Identity already exists"
        },
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(resp_json));
}
