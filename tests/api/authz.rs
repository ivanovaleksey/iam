use actix_web::HttpMessage;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

use shared;

#[test]
fn test_authorization() {
    use iam::models::*;
    use iam::schema::*;

    let shared::Server { mut srv, pool } = shared::build_server();

    let account_id = Uuid::new_v4();
    let namespace_id = Uuid::new_v4();

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

        diesel::insert_into(abac_subject_attr::table)
            .values((
                abac_subject_attr::namespace_id.eq(namespace.id),
                abac_subject_attr::subject_id.eq(account.id),
                abac_subject_attr::value.eq("role:client".to_owned()),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_object_attr::table)
            .values((
                abac_object_attr::namespace_id.eq(namespace.id),
                abac_object_attr::object_id.eq("room"),
                abac_object_attr::value.eq("type:room"),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("create"),
                abac_action_attr::value.eq("access:owner"),
            ))
            .execute(&conn)
            .unwrap();
        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("read"),
                abac_action_attr::value.eq("access:owner"),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_policy::table)
            .values((
                abac_policy::namespace_id.eq(namespace.id),
                abac_policy::subject_value.eq("role:client"),
                abac_policy::object_value.eq("type:room"),
                abac_policy::action_value.eq("access:owner"),
                abac_policy::issued_at.eq(diesel::dsl::now),
            ))
            .execute(&conn)
            .unwrap();
    }

    let json = json!({
        "jsonrpc": "2.0",
        "method": "auth",
        "params": [{
            "namespace_ids": [namespace_id],
            "subject": account_id,
            "object": "room",
            "action": "create",
        }],
        "id": "qwerty",
    });
    let req = srv.get()
        .body(serde_json::to_string(&json).unwrap())
        .unwrap();

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, r#"{"jsonrpc":"2.0","result":true,"id":"qwerty"}"#);

    let json = json!({
        "jsonrpc": "2.0",
        "method": "auth",
        "params": [{
            "namespace_ids": [namespace_id],
            "subject": account_id,
            "object": "room",
            "action": "delete",
        }],
        "id": "qwerty",
    });
    let req = srv.get()
        .body(serde_json::to_string(&json).unwrap())
        .unwrap();

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, r#"{"jsonrpc":"2.0","result":false,"id":"qwerty"}"#);
}
