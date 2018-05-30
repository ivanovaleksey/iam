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
            let (account, namespace) = before_each(&conn);

            diesel::insert_into(abac_subject_attr::table)
                .values((
                    abac_subject_attr::namespace_id.eq(namespace.id),
                    abac_subject_attr::subject_id.eq(account.id),
                    abac_subject_attr::key.eq("role"),
                    abac_subject_attr::value.eq("admin"),
                ))
                .execute(&conn)
                .unwrap();

            diesel::insert_into(abac_subject_attr::table)
                .values((
                    abac_subject_attr::namespace_id.eq(namespace.id),
                    abac_subject_attr::subject_id.eq(account.id),
                    abac_subject_attr::key.eq("role"),
                    abac_subject_attr::value.eq("client"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_subject_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND subject_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420"
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
                    "key": "owner:namespace",
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "subject_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
                    "value": "bab37008-3dc5-492c-af73-80c241241d71"
                },
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
        assert_eq!(body, shared::strip_json(resp_json));
    }

    #[test]
    fn anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (account, namespace) = before_each(&conn);

            diesel::insert_into(abac_subject_attr::table)
                .values((
                    abac_subject_attr::namespace_id.eq(namespace.id),
                    abac_subject_attr::subject_id.eq(account.id),
                    abac_subject_attr::key.eq("role"),
                    abac_subject_attr::value.eq("admin"),
                ))
                .execute(&conn)
                .unwrap();

            diesel::insert_into(abac_subject_attr::table)
                .values((
                    abac_subject_attr::namespace_id.eq(namespace.id),
                    abac_subject_attr::subject_id.eq(account.id),
                    abac_subject_attr::key.eq("role"),
                    abac_subject_attr::value.eq("client"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_subject_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND subject_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420"
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

        let account = shared::db::create_iam_account(conn);
        let namespace = shared::db::create_iam_namespace(conn, account.id);

        (account, namespace)
    }

    #[test]
    fn auth_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = pool.get().expect("Failed to get connection from pool");
            let (account, namespace) = before_each(&conn);

            diesel::insert_into(abac_subject_attr::table)
                .values((
                    abac_subject_attr::namespace_id.eq(namespace.id),
                    abac_subject_attr::subject_id.eq(account.id),
                    abac_subject_attr::key.eq("role"),
                    abac_subject_attr::value.eq("admin"),
                ))
                .execute(&conn)
                .unwrap();

            diesel::insert_into(abac_subject_attr::table)
                .values((
                    abac_subject_attr::namespace_id.eq(namespace.id),
                    abac_subject_attr::subject_id.eq(account.id),
                    abac_subject_attr::key.eq("role"),
                    abac_subject_attr::value.eq("client"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_subject_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND subject_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420"
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
            let (account, namespace) = before_each(&conn);

            diesel::insert_into(abac_subject_attr::table)
                .values((
                    abac_subject_attr::namespace_id.eq(namespace.id),
                    abac_subject_attr::subject_id.eq(account.id),
                    abac_subject_attr::key.eq("role"),
                    abac_subject_attr::value.eq("admin"),
                ))
                .execute(&conn)
                .unwrap();

            diesel::insert_into(abac_subject_attr::table)
                .values((
                    abac_subject_attr::namespace_id.eq(namespace.id),
                    abac_subject_attr::subject_id.eq(account.id),
                    abac_subject_attr::key.eq("role"),
                    abac_subject_attr::value.eq("client"),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req_json = r#"{
            "jsonrpc": "2.0",
            "method": "abac_subject_attr.list",
            "params": [{
                "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND subject_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420"
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
