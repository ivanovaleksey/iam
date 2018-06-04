use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::models::*;
use iam::schema::*;

use shared;

lazy_static! {
    static ref FOXFORD_NAMESPACE_ID: Uuid = Uuid::new_v4();
    static ref FOXFORD_USER_1_ID: Uuid = Uuid::new_v4();
    static ref FOXFORD_USER_2_ID: Uuid = Uuid::new_v4();
    static ref NETOLOGY_NAMESPACE_ID: Uuid = Uuid::new_v4();
    static ref NETOLOGY_USER_ID: Uuid = Uuid::new_v4();
    static ref USER_1_ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref USER_2_ACCOUNT_ID: Uuid = Uuid::new_v4();
}

fn before_each(conn: &PgConnection) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let foxford_account = diesel::insert_into(account::table)
        .values((account::id.eq(Uuid::new_v4()), account::enabled.eq(true)))
        .get_result::<Account>(conn)
        .unwrap();

    let netology_account = diesel::insert_into(account::table)
        .values((account::id.eq(Uuid::new_v4()), account::enabled.eq(true)))
        .get_result::<Account>(conn)
        .unwrap();

    let user_1_account = diesel::insert_into(account::table)
        .values((
            account::id.eq(*USER_1_ACCOUNT_ID),
            account::enabled.eq(true),
        ))
        .get_result::<Account>(conn)
        .unwrap();

    let user_2_account = diesel::insert_into(account::table)
        .values((
            account::id.eq(*USER_2_ACCOUNT_ID),
            account::enabled.eq(true),
        ))
        .get_result::<Account>(conn)
        .unwrap();

    let foxford_namespace = diesel::insert_into(namespace::table)
        .values((
            namespace::id.eq(*FOXFORD_NAMESPACE_ID),
            namespace::label.eq("foxford.ru"),
            namespace::account_id.eq(foxford_account.id),
            namespace::enabled.eq(true),
        ))
        .get_result::<Namespace>(conn)
        .unwrap();

    let netology_namespace = diesel::insert_into(namespace::table)
        .values((
            namespace::id.eq(*NETOLOGY_NAMESPACE_ID),
            namespace::label.eq("netology.ru"),
            namespace::account_id.eq(netology_account.id),
            namespace::enabled.eq(true),
        ))
        .get_result::<Namespace>(conn)
        .unwrap();

    diesel::insert_into(identity::table)
        .values((
            identity::provider.eq(foxford_namespace.id),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_1_ID.to_string()),
            identity::account_id.eq(user_1_account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(identity::table)
        .values((
            identity::provider.eq(foxford_namespace.id),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_2_ID.to_string()),
            identity::account_id.eq(user_2_account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(identity::table)
        .values((
            identity::provider.eq(netology_namespace.id),
            identity::label.eq("oauth2"),
            identity::uid.eq(NETOLOGY_USER_ID.to_string()),
            identity::account_id.eq(user_1_account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
        ))
        .execute(conn)
        .unwrap();
}

#[test]
fn without_filter() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        before_each(&conn);
    }

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "identity.list",
        "params": [{
            "fq": ""
        }],
        "id": "qwerty"
    }"#;
    let req = shared::build_rpc_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp_template = r#"{
        "jsonrpc": "2.0",
        "result": [
            {
                "account_id": "USER_1_ACCOUNT_ID",
                "created_at": "2018-06-02T08:40:00",
                "label": "oauth2",
                "provider": "FOXFORD_NAMESPACE_ID",
                "uid": "FOXFORD_USER_1_ID"
            },
            {
                "account_id": "USER_2_ACCOUNT_ID",
                "created_at": "2018-06-02T08:40:00",
                "label": "oauth2",
                "provider": "FOXFORD_NAMESPACE_ID",
                "uid": "FOXFORD_USER_2_ID"
            },
            {
                "account_id": "USER_1_ACCOUNT_ID",
                "created_at": "2018-06-02T08:40:00",
                "label": "oauth2",
                "provider": "NETOLOGY_NAMESPACE_ID",
                "uid": "NETOLOGY_USER_ID"
            }
        ],
        "id": "qwerty"
    }"#;
    let resp_json = resp_template
        .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
        .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
        .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
        .replace("NETOLOGY_NAMESPACE_ID", &NETOLOGY_NAMESPACE_ID.to_string())
        .replace("NETOLOGY_USER_ID", &NETOLOGY_USER_ID.to_string())
        .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string())
        .replace("USER_2_ACCOUNT_ID", &USER_2_ACCOUNT_ID.to_string());
    assert_eq!(body, shared::strip_json(&resp_json));
}

#[test]
fn with_filter_by_provider() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        before_each(&conn);
    }

    let req_template = r#"{
        "jsonrpc": "2.0",
        "method": "identity.list",
        "params": [{
            "fq": "provider:FOXFORD_NAMESPACE_ID"
        }],
        "id": "qwerty"
    }"#;
    let req_json = req_template.replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

    let req = shared::build_rpc_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp_template = r#"{
        "jsonrpc": "2.0",
        "result": [
            {
                "account_id": "USER_1_ACCOUNT_ID",
                "created_at": "2018-06-02T08:40:00",
                "label": "oauth2",
                "provider": "FOXFORD_NAMESPACE_ID",
                "uid": "FOXFORD_USER_1_ID"
            },
            {
                "account_id": "USER_2_ACCOUNT_ID",
                "created_at": "2018-06-02T08:40:00",
                "label": "oauth2",
                "provider": "FOXFORD_NAMESPACE_ID",
                "uid": "FOXFORD_USER_2_ID"
            }
        ],
        "id": "qwerty"
    }"#;
    let resp_json = resp_template
        .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
        .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
        .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
        .replace("USER_1_ACCOUNT_ID", &USER_1_ACCOUNT_ID.to_string())
        .replace("USER_2_ACCOUNT_ID", &USER_2_ACCOUNT_ID.to_string());
    assert_eq!(body, shared::strip_json(&resp_json));
}

#[test]
fn with_filter_by_provider_and_account() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        before_each(&conn);
    }

    let req_template = r#"{
        "jsonrpc": "2.0",
        "method": "identity.list",
        "params": [{
            "fq": "provider:FOXFORD_NAMESPACE_ID AND account_id:USER_2_ACCOUNT_ID"
        }],
        "id": "qwerty"
    }"#;
    let req_json = req_template
        .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
        .replace("USER_2_ACCOUNT_ID", &USER_2_ACCOUNT_ID.to_string());

    let req = shared::build_rpc_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp_template = r#"{
        "jsonrpc": "2.0",
        "result": [
            {
                "account_id": "USER_2_ACCOUNT_ID",
                "created_at": "2018-06-02T08:40:00",
                "label": "oauth2",
                "provider": "FOXFORD_NAMESPACE_ID",
                "uid": "FOXFORD_USER_2_ID"
            }
        ],
        "id": "qwerty"
    }"#;
    let resp_json = resp_template
        .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
        .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
        .replace("USER_2_ACCOUNT_ID", &USER_2_ACCOUNT_ID.to_string());
    assert_eq!(body, shared::strip_json(&resp_json));
}
