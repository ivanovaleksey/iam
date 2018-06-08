use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

use iam::models::prelude::*;
use iam::schema::*;

use shared;

lazy_static! {
    static ref FOXFORD_NAMESPACE_ID: Uuid = Uuid::new_v4();
    static ref FOXFORD_USER_ID: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID: Uuid = Uuid::new_v4();
}

fn before_each(conn: &PgConnection) -> Account {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let foxford_account = diesel::insert_into(account::table)
        .values((account::id.eq(Uuid::new_v4()), account::enabled.eq(true)))
        .get_result::<Account>(conn)
        .unwrap();

    let user_account = diesel::insert_into(account::table)
        .values((account::id.eq(*USER_ACCOUNT_ID), account::enabled.eq(true)))
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

    diesel::insert_into(identity::table)
        .values((
            identity::provider.eq(*FOXFORD_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_ID.to_string()),
            identity::account_id.eq(user_account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
        ))
        .execute(conn)
        .unwrap();

    user_account
}

#[test]
fn delete_last_identity() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        let _ = before_each(&conn);
    }

    let req_json = json!({
        "jsonrpc": "2.0",
        "method": "identity.delete",
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
    let resp_template = r#"{
        "jsonrpc": "2.0",
        "result": {
            "account_id": "USER_ACCOUNT_ID",
            "created_at": "2018-06-02T08:40:00",
            "label": "oauth2",
            "provider": "FOXFORD_NAMESPACE_ID",
            "uid": "FOXFORD_USER_ID"
        },
        "id": "qwerty"
    }"#;
    let resp_json = resp_template
        .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
        .replace("FOXFORD_USER_ID", &FOXFORD_USER_ID.to_string())
        .replace("USER_ACCOUNT_ID", &USER_ACCOUNT_ID.to_string());
    assert_eq!(body, shared::strip_json(&resp_json));

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

    let conn = pool.get().expect("Failed to get connection from pool");
    let result = account::table
        .find(*USER_ACCOUNT_ID)
        .get_result::<Account>(&conn);

    println!("{:?}", result);

    if let Err(diesel::result::Error::NotFound) = result {
        assert!(true);
    } else {
        assert!(false);
    }
}

#[test]
fn delete_nonlast_identity() {
    let shared::Server { mut srv, pool } = shared::build_server();

    let netology_namespace_id = Uuid::new_v4();
    let netology_user_id = Uuid::new_v4();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        let user_account = before_each(&conn);

        let netology_account = diesel::insert_into(account::table)
            .values((account::id.eq(Uuid::new_v4()), account::enabled.eq(true)))
            .get_result::<Account>(&conn)
            .unwrap();

        let netology_namespace = diesel::insert_into(namespace::table)
            .values((
                namespace::id.eq(netology_namespace_id),
                namespace::label.eq("netology.ru"),
                namespace::account_id.eq(netology_account.id),
                namespace::enabled.eq(true),
            ))
            .get_result::<Namespace>(&conn)
            .unwrap();

        diesel::insert_into(identity::table)
            .values((
                identity::provider.eq(netology_namespace.id),
                identity::label.eq("oauth2"),
                identity::uid.eq(netology_user_id.to_string()),
                identity::account_id.eq(user_account.id),
                identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
            ))
            .execute(&conn)
            .unwrap();
    }

    let req_json = json!({
        "jsonrpc": "2.0",
        "method": "identity.delete",
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
    let resp_template = r#"{
        "jsonrpc": "2.0",
        "result": {
            "account_id": "USER_ACCOUNT_ID",
            "created_at": "2018-06-02T08:40:00",
            "label": "oauth2",
            "provider": "FOXFORD_NAMESPACE_ID",
            "uid": "FOXFORD_USER_ID"
        },
        "id": "qwerty"
    }"#;
    let resp_json = resp_template
        .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
        .replace("FOXFORD_USER_ID", &FOXFORD_USER_ID.to_string())
        .replace("USER_ACCOUNT_ID", &USER_ACCOUNT_ID.to_string());
    assert_eq!(body, shared::strip_json(&resp_json));

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

    let conn = pool.get().expect("Failed to get connection from pool");
    let result = account::table
        .find(*USER_ACCOUNT_ID)
        .get_result::<Account>(&conn);

    assert!(result.is_ok());
}
