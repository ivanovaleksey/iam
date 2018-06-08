use diesel::prelude::*;
use jsonrpc;
use serde_json;

use iam::models::*;

use shared;

mod with_namespace_ownership {
    use super::*;
    use actix_web::HttpMessage;

    fn before_each(conn: &PgConnection) -> (Account, Namespace) {
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(conn);
        let namespace = shared::db::create_iam_namespace(conn, account.id);

        shared::db::grant_namespace_ownership(&conn, namespace.id, account.id);

        (account, namespace)
    }

    #[test]
    fn auth_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let _ = before_each(&conn);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.create",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_key": "role",
                "subject_value": "client",
                "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "object_key": "type",
                "object_value": "identity",
                "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_key": "action",
                "action_value": "*"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_auth_request(&srv, req_json.to_owned(), None);

        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        if let Ok(resp) = serde_json::from_slice::<jsonrpc::Success>(&body) {
            let policy: AbacPolicy = serde_json::from_value(resp.result).unwrap();

            assert_eq!(policy.namespace_id, *shared::IAM_NAMESPACE_ID);
            assert_eq!(policy.subject_namespace_id, *shared::IAM_NAMESPACE_ID);
            assert_eq!(policy.subject_key, "role".to_owned());
            assert_eq!(policy.subject_value, "client".to_owned());
            assert_eq!(policy.object_namespace_id, *shared::IAM_NAMESPACE_ID);
            assert_eq!(policy.object_key, "type".to_owned());
            assert_eq!(policy.object_value, "identity".to_owned());
            assert_eq!(policy.action_namespace_id, *shared::IAM_NAMESPACE_ID);
            assert_eq!(policy.action_key, "action".to_owned());
            assert_eq!(policy.action_value, "*".to_owned());
        } else {
            panic!("{:?}", body);
        }
    }

    #[test]
    fn anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let _ = before_each(&conn);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.create",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_key": "role",
                "subject_value": "client",
                "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "object_key": "type",
                "object_value": "identity",
                "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_key": "action",
                "action_value": "*"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_anonymous_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
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
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(conn);
        let namespace = shared::db::create_iam_namespace(conn, account.id);

        (account, namespace)
    }

    #[test]
    fn auth_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let _ = before_each(&conn);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.create",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_key": "role",
                "subject_value": "client",
                "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "object_key": "type",
                "object_value": "identity",
                "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_key": "action",
                "action_value": "*"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_auth_request(&srv, req_json.to_owned(), None);

        let resp = srv.execute(req.send()).unwrap();
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
            let _ = before_each(&conn);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.create",
            "params": [{
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "subject_key": "role",
                "subject_value": "client",
                "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "object_key": "type",
                "object_value": "identity",
                "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "action_key": "action",
                "action_value": "*"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_anonymous_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
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
