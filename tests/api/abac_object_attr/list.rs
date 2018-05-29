use diesel;
use diesel::prelude::*;

use iam::models::*;
use iam::schema::*;

use shared;

mod with_namespace_ownership {
    use super::*;
    use actix_web::HttpMessage;

    fn before_each(conn: &PgConnection) -> (Account, Namespace) {
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(&conn);
        let namespace = shared::db::create_iam_namespace(conn, account.id);

        shared::db::grant_namespace_ownership(&conn, namespace.id, account.id);

        (account, namespace)
    }

    #[test]
    fn auth_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            diesel::insert_into(abac_object_attr::table)
                .values((
                    abac_object_attr::namespace_id.eq(namespace.id),
                    abac_object_attr::object_id.eq("room"),
                    abac_object_attr::key.eq("type"),
                    abac_object_attr::value.eq("room"),
                ))
                .execute(&conn)
                .unwrap();

            diesel::insert_into(abac_object_attr::table)
                .values((
                    abac_object_attr::namespace_id.eq(namespace.id),
                    abac_object_attr::object_id.eq("namespace"),
                    abac_object_attr::key.eq("type"),
                    abac_object_attr::value.eq("namespace"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_object_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:room"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_rpc_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
        assert!(resp.status().is_success());

        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "key": "type",
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "object_id": "room",
                    "value": "room"
                }
            ],
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(resp_json));

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_object_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:namespace"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_rpc_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
        assert!(resp.status().is_success());

        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "key": "type",
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "object_id": "namespace",
                    "value": "namespace"
                }
            ],
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(resp_json));
    }

    #[test]
    fn anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            diesel::insert_into(abac_object_attr::table)
                .values((
                    abac_object_attr::namespace_id.eq(namespace.id),
                    abac_object_attr::object_id.eq("room"),
                    abac_object_attr::key.eq("type"),
                    abac_object_attr::value.eq("room"),
                ))
                .execute(&conn)
                .unwrap();

            diesel::insert_into(abac_object_attr::table)
                .values((
                    abac_object_attr::namespace_id.eq(namespace.id),
                    abac_object_attr::object_id.eq("namespace"),
                    abac_object_attr::key.eq("type"),
                    abac_object_attr::value.eq("namespace"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_object_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:namespace"
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
                "code": 403,
                "message": "Forbidden"
            },
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(resp_json));
    }
}

mod without_namespace_ownership {
    use super::*;
    use actix_web::HttpMessage;

    fn before_each(conn: &PgConnection) -> (Account, Namespace) {
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(&conn);
        let namespace = shared::db::create_iam_namespace(conn, account.id);

        (account, namespace)
    }

    #[test]
    fn auth_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            diesel::insert_into(abac_object_attr::table)
                .values((
                    abac_object_attr::namespace_id.eq(namespace.id),
                    abac_object_attr::object_id.eq("room"),
                    abac_object_attr::key.eq("type"),
                    abac_object_attr::value.eq("room"),
                ))
                .execute(&conn)
                .unwrap();

            diesel::insert_into(abac_object_attr::table)
                .values((
                    abac_object_attr::namespace_id.eq(namespace.id),
                    abac_object_attr::object_id.eq("namespace"),
                    abac_object_attr::key.eq("type"),
                    abac_object_attr::value.eq("namespace"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_object_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:room"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_rpc_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
        assert!(resp.status().is_success());

        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": 403,
                "message": "Forbidden"
            },
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(resp_json));
    }

    #[test]
    fn anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            diesel::insert_into(abac_object_attr::table)
                .values((
                    abac_object_attr::namespace_id.eq(namespace.id),
                    abac_object_attr::object_id.eq("room"),
                    abac_object_attr::key.eq("type"),
                    abac_object_attr::value.eq("room"),
                ))
                .execute(&conn)
                .unwrap();

            diesel::insert_into(abac_object_attr::table)
                .values((
                    abac_object_attr::namespace_id.eq(namespace.id),
                    abac_object_attr::object_id.eq("namespace"),
                    abac_object_attr::key.eq("type"),
                    abac_object_attr::value.eq("namespace"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_object_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:room"
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
                "code": 403,
                "message": "Forbidden"
            },
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(resp_json));
    }
}
