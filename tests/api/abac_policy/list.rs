use chrono::NaiveDate;
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
        conn.begin_test_transaction()
            .expect("Failed to begin transaction");

        let account = shared::db::create_iam_account(conn);
        let namespace = shared::db::create_iam_namespace(conn, account.id);

        shared::db::grant_namespace_ownership(&conn, namespace.id, account.id);

        (account, namespace)
    }

    #[test]
    fn auth_request_with_filter() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            create_policies(&conn, namespace.id);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_rpc_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "action_key": "action",
                    "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "action_value": "*",
                    "created_at":"2018-05-29T07:15:00",
                    "expired_at": null,
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "not_before": null,
                    "object_key": "belongs_to:namespace",
                    "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "object_value": "bab37008-3dc5-492c-af73-80c241241d71",
                    "subject_key": "owner:namespace",
                    "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "subject_value": "bab37008-3dc5-492c-af73-80c241241d71"
                },
                {
                    "action_key": "action",
                    "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "action_value": "*",
                    "created_at":"2018-05-29T07:15:00",
                    "expired_at": null,
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "not_before": null,
                    "object_key": "type",
                    "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "object_value": "namespace",
                    "subject_key": "role",
                    "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "subject_value": "admin"
                },
                {
                    "action_key": "action",
                    "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "action_value": "*",
                    "created_at":"2018-05-29T07:15:00",
                    "expired_at": null,
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "not_before": null,
                    "object_key": "type",
                    "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "object_value": "identity",
                    "subject_key": "role",
                    "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "subject_value": "client"
                }
            ],
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn auth_request_without_filter() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            create_policies(&conn, namespace.id);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.list",
            "params": [{
                "fq": ""
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_rpc_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32602,
                "message": "Invalid params: missing field `namespace_id`."
            },
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn anonymous_request_with_filter() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            create_policies(&conn, namespace.id);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71"
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

    #[test]
    fn anonymous_request_without_filter() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (_account, namespace) = before_each(&conn);

            create_policies(&conn, namespace.id);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.list",
            "params": [{
                "fq": ""
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_anonymous_request(&srv, req_json.to_owned());

        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": -32602,
                "message": "Invalid params: missing field `namespace_id`."
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
            let (_account, namespace) = before_each(&conn);

            create_policies(&conn, namespace.id);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71"
            }],
            "id": "qwerty"
        }"#;
        let req = shared::build_rpc_request(&srv, req_json.to_owned());

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
            let (_account, namespace) = before_each(&conn);

            create_policies(&conn, namespace.id);
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_policy.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71"
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

fn create_policies(conn: &PgConnection, namespace_id: Uuid) {
    diesel::insert_into(abac_policy::table)
        .values((
            abac_policy::namespace_id.eq(namespace_id),
            abac_policy::subject_namespace_id.eq(namespace_id),
            abac_policy::subject_key.eq("role"),
            abac_policy::subject_value.eq("admin"),
            abac_policy::object_namespace_id.eq(namespace_id),
            abac_policy::object_key.eq("type"),
            abac_policy::object_value.eq("namespace"),
            abac_policy::action_namespace_id.eq(namespace_id),
            abac_policy::action_key.eq("action"),
            abac_policy::action_value.eq("*"),
            abac_policy::created_at.eq(NaiveDate::from_ymd(2018, 5, 29).and_hms(7, 15, 0)),
        ))
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_policy::table)
        .values((
            abac_policy::namespace_id.eq(namespace_id),
            abac_policy::subject_namespace_id.eq(namespace_id),
            abac_policy::subject_key.eq("role"),
            abac_policy::subject_value.eq("client"),
            abac_policy::object_namespace_id.eq(namespace_id),
            abac_policy::object_key.eq("type"),
            abac_policy::object_value.eq("identity"),
            abac_policy::action_namespace_id.eq(namespace_id),
            abac_policy::action_key.eq("action"),
            abac_policy::action_value.eq("*"),
            abac_policy::created_at.eq(NaiveDate::from_ymd(2018, 5, 29).and_hms(7, 15, 0)),
        ))
        .execute(conn)
        .unwrap();
}