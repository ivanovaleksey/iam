use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::models::*;
use iam::schema::*;

use shared;

mod with_namespace_ownership {
    use super::*;
    use actix_web::HttpMessage;

    fn before_each(conn: &PgConnection) -> (Account, Namespace) {
        let account_id = Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap();

        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = diesel::insert_into(account::table)
            .values((account::id.eq(account_id), account::enabled.eq(true)))
            .get_result::<Account>(conn)
            .unwrap();

        let namespace = shared::db::create_iam_namespace(conn, account.id);

        shared::db::grant_namespace_ownership(&conn, namespace.id, account.id);

        (account, namespace)
    }

    #[test]
    fn with_existing_record() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            diesel::insert_into(abac_action_attr::table)
                .values((
                    abac_action_attr::namespace_id.eq(namespace.id),
                    abac_action_attr::action_id.eq("create"),
                    abac_action_attr::key.eq("access"),
                    abac_action_attr::value.eq("*"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_action_attr.read",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_id": "create",
                "key": "access",
                "value": "*"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_rpc_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
        assert!(resp.status().is_success());

        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "result": {
                "action_id": "create",
                "key": "access",
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "value": "*"
            },
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_nonexisting_record() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, _namespace) = before_each(&conn);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_action_attr.read",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_id": "create",
                "key": "access",
                "value": "*"
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
                "code": 404,
                "message": "NotFound"
            },
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, _namespace) = before_each(&conn);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_action_attr.read",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_id": "create",
                "key": "access",
                "value": "*"
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
        assert_eq!(body, shared::strip_json(&resp_json));
    }
}

mod without_namespace_ownership {
    use super::*;
    use actix_web::HttpMessage;

    fn before_each(conn: &PgConnection) -> (Account, Namespace) {
        let account_id = Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap();

        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = diesel::insert_into(account::table)
            .values((account::id.eq(account_id), account::enabled.eq(true)))
            .get_result::<Account>(conn)
            .unwrap();

        let namespace = shared::db::create_iam_namespace(conn, account.id);

        (account, namespace)
    }

    #[test]
    fn with_existing_record() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            diesel::insert_into(abac_action_attr::table)
                .values((
                    abac_action_attr::namespace_id.eq(namespace.id),
                    abac_action_attr::action_id.eq("create"),
                    abac_action_attr::key.eq("access"),
                    abac_action_attr::value.eq("*"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_action_attr.read",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_id": "create",
                "key": "access",
                "value": "*"
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
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_nonexisting_record() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, _namespace) = before_each(&conn);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_action_attr.read",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_id": "create",
                "key": "access",
                "value": "*"
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
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, _namespace) = before_each(&conn);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_action_attr.read",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_id": "create",
                "key": "access",
                "value": "*"
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
        assert_eq!(body, shared::strip_json(&resp_json));
    }
}
