use actix_web::HttpMessage;
use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::models::*;
use iam::schema::*;

use shared;

#[test]
fn test() {
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

        diesel::insert_into(abac_subject_attr::table)
            .values((
                abac_subject_attr::namespace_id.eq(namespace.id),
                abac_subject_attr::subject_id.eq(account_id),
                abac_subject_attr::key.eq("role"),
                abac_subject_attr::value.eq("admin"),
            ))
            .execute(&conn)
            .unwrap();

        diesel::insert_into(abac_subject_attr::table)
            .values((
                abac_subject_attr::namespace_id.eq(namespace.id),
                abac_subject_attr::subject_id.eq(account_id),
                abac_subject_attr::key.eq("role"),
                abac_subject_attr::value.eq("client"),
            ))
            .execute(&conn)
            .unwrap();
    }

    let json = r#"{
        "jsonrpc": "2.0",
        "method": "abac_subject_attr.list",
        "params": [{
            "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND subject_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420"
        }],
        "id": "qwerty"
    }"#;
    let req = srv.post().body(json).unwrap();

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let json = r#"{
        "jsonrpc": "2.0",
        "result": [
            {
                "key": "role",
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
                "value": "admin"
            },
            {
                "key": "role",
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
                "value": "client"
            }
        ],
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(&json));
}
