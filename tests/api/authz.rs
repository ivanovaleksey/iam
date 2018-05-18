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
                abac_subject_attr::key.eq("role".to_owned()),
                abac_subject_attr::value.eq("client".to_owned()),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_object_attr::table)
            .values((
                abac_object_attr::namespace_id.eq(namespace.id),
                abac_object_attr::object_id.eq("room"),
                abac_object_attr::key.eq("type"),
                abac_object_attr::value.eq("room"),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("create"),
                abac_action_attr::key.eq("access"),
                abac_action_attr::value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();
        diesel::insert_into(abac_action_attr::table)
            .values((
                abac_action_attr::namespace_id.eq(namespace.id),
                abac_action_attr::action_id.eq("read"),
                abac_action_attr::key.eq("access"),
                abac_action_attr::value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();

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
                abac_policy::action_key.eq("access"),
                abac_policy::action_value.eq("*"),
            ))
            .execute(&conn)
            .unwrap();
    }

    let json = json!({
        "jsonrpc": "2.0",
        "method": "authorize",
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
        "method": "authorize",
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
