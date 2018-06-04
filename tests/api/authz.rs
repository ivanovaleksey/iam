use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

use iam::models::*;
use iam::schema::*;

use shared;

fn before_each(conn: &PgConnection) -> (Account, Namespace) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let account = shared::db::create_iam_account(&conn);
    let namespace = shared::db::create_iam_namespace(conn, account.id);

    diesel::insert_into(abac_subject_attr::table)
        .values((
            abac_subject_attr::namespace_id.eq(namespace.id),
            abac_subject_attr::subject_id.eq(account.id),
            abac_subject_attr::key.eq("role".to_owned()),
            abac_subject_attr::value.eq("client".to_owned()),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_object_attr::table)
        .values((
            abac_object_attr::namespace_id.eq(namespace.id),
            abac_object_attr::object_id.eq("room"),
            abac_object_attr::key.eq("type"),
            abac_object_attr::value.eq("room"),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_action_attr::table)
        .values((
            abac_action_attr::namespace_id.eq(namespace.id),
            abac_action_attr::action_id.eq("create"),
            abac_action_attr::key.eq("action"),
            abac_action_attr::value.eq("*"),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_action_attr::table)
        .values((
            abac_action_attr::namespace_id.eq(namespace.id),
            abac_action_attr::action_id.eq("read"),
            abac_action_attr::key.eq("action"),
            abac_action_attr::value.eq("*"),
        ))
        .execute(conn)
        .unwrap();

    (account, namespace)
}

#[test]
fn with_permission() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        let (_account, namespace) = before_each(&conn);

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_namespace_id.eq(namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("client"),
                abac_policy::object_namespace_id.eq(namespace.id),
                abac_policy::object_key.eq("type"),
                abac_policy::object_value.eq("room"),
                abac_policy::action_namespace_id.eq(namespace.id),
                abac_policy::action_key.eq("action"),
                abac_policy::action_value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();
    }

    let req_json = json!({
        "jsonrpc": "2.0",
        "method": "authorize",
        "params": [{
            "namespace_ids": ["bab37008-3dc5-492c-af73-80c241241d71"],
            "subject": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "object": "room",
            "action": "create",
        }],
        "id": "qwerty",
    });
    let req = shared::build_anonymous_request(&srv, serde_json::to_string(&req_json).unwrap());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "result": true,
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(resp_json));
}

#[test]
fn without_permission() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        let _ = before_each(&conn);
    }

    let req_json = json!({
        "jsonrpc": "2.0",
        "method": "authorize",
        "params": [{
            "namespace_ids": ["bab37008-3dc5-492c-af73-80c241241d71"],
            "subject": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "object": "room",
            "action": "read",
        }],
        "id": "qwerty",
    });
    let req = shared::build_anonymous_request(&srv, serde_json::to_string(&req_json).unwrap());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "result": false,
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(resp_json));
}

#[test]
fn reuse_action_from_another_namespace() {
    let shared::Server { mut srv, pool } = shared::build_server();

    let foxford_account_id = Uuid::new_v4();
    let foxford_namespace_id = Uuid::new_v4();
    let user_account_id = Uuid::new_v4();

    {
        let conn = pool.get().expect("Failed to get connection from pool");
        let (_iam_account, iam_namespace) = before_each(&conn);

        let foxford_account = diesel::insert_into(account::table)
            .values((
                account::id.eq(foxford_account_id),
                account::enabled.eq(true),
            ))
            .get_result::<Account>(&conn)
            .unwrap();

        let foxford_namespace = diesel::insert_into(namespace::table)
            .values((
                namespace::id.eq(foxford_namespace_id),
                namespace::label.eq("foxford.ru"),
                namespace::account_id.eq(foxford_account.id),
                namespace::enabled.eq(true),
                namespace::created_at.eq(NaiveDate::from_ymd(2018, 6, 4).and_hms(17, 30, 0)),
            ))
            .get_result::<Namespace>(&conn)
            .unwrap();

        let user_account = diesel::insert_into(account::table)
            .values((account::id.eq(user_account_id), account::enabled.eq(true)))
            .get_result::<Account>(&conn)
            .unwrap();

        diesel::insert_into(abac_subject_attr::table)
            .values((
                abac_subject_attr::namespace_id.eq(foxford_namespace.id),
                abac_subject_attr::subject_id.eq(user_account.id),
                abac_subject_attr::key.eq("role".to_owned()),
                abac_subject_attr::value.eq("user".to_owned()),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_object_attr::table)
            .values((
                abac_object_attr::namespace_id.eq(foxford_namespace.id),
                abac_object_attr::object_id.eq("webinar.1".to_owned()),
                abac_object_attr::key.eq("group".to_owned()),
                abac_object_attr::value.eq("A".to_owned()),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(foxford_namespace.id),
                abac_policy::subject_namespace_id.eq(foxford_namespace.id),
                abac_policy::subject_key.eq("role"),
                abac_policy::subject_value.eq("user"),
                abac_policy::object_namespace_id.eq(foxford_namespace.id),
                abac_policy::object_key.eq("group"),
                abac_policy::object_value.eq("A"),
                abac_policy::action_namespace_id.eq(iam_namespace.id),
                abac_policy::action_key.eq("action"),
                abac_policy::action_value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();
    }

    let json = json!({
        "jsonrpc": "2.0",
        "method": "authorize",
        "params": [{
            "namespace_ids": [format!("{}", foxford_namespace_id)],
            "subject": format!("{}", user_account_id),
            "object": "webinar.1",
            "action": "read",
        }],
        "id": "qwerty",
    });
    let req = shared::build_anonymous_request(&srv, serde_json::to_string(&json).unwrap());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "result": true,
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(resp_json));
}
