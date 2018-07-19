use diesel::prelude::*;
use serde_json;

use iam::models::{Account, Namespace};
use iam::schema::namespace;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, NETOLOGY_ACCOUNT_ID};

#[must_use]
fn before_each_1(conn: &PgConnection) -> (Account, Namespace) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    let netology_account = create_account(conn, AccountKind::Netology);
    let _netology_namespace = create_namespace(conn, NamespaceKind::Netology(netology_account.id));

    (iam_account, iam_namespace)
}

mod with_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> Namespace {
        let _ = before_each_1(conn);

        let foxford_account = create_account(conn, AccountKind::Foxford);
        let foxford_namespace = create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

        foxford_namespace
    }

    mod with_admin {
        use super::*;

        #[test]
        fn can_update_label() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let changeset = build_request("foxford-new.ru");
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&changeset).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let template = r#"{
                "jsonrpc": "2.0",
                "result": {
                    "data": {
                        "account_id": "FOXFORD_ACCOUNT_ID",
                        "created_at": "2018-05-30T08:40:00Z",
                        "label": "foxford-new.ru"
                    },
                    "id": "FOXFORD_NAMESPACE_ID"
                },
                "id": "qwerty"
            }"#;

            let json = template
                .replace("FOXFORD_ACCOUNT_ID", &FOXFORD_ACCOUNT_ID.to_string())
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

            assert_eq!(body, shared::strip_json(&json));

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn).label, "foxford-new.ru");
            }
        }
    }

    mod with_own_client {
        use super::*;

        #[test]
        fn can_update_label() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let changeset = build_request("foxford-new.ru");
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&changeset).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let template = r#"{
                "jsonrpc": "2.0",
                "result": {
                    "data": {
                        "account_id": "FOXFORD_ACCOUNT_ID",
                        "created_at": "2018-05-30T08:40:00Z",
                        "label": "foxford-new.ru"
                    },
                    "id": "FOXFORD_NAMESPACE_ID"
                },
                "id": "qwerty"
            }"#;

            let json = template
                .replace("FOXFORD_ACCOUNT_ID", &FOXFORD_ACCOUNT_ID.to_string())
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

            assert_eq!(body, shared::strip_json(&json));

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn).label, "foxford-new.ru");
            }
        }
    }

    mod with_alien_client {
        use super::*;

        #[test]
        fn cannot_update_label() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let changeset = build_request("foxford-new.ru");
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&changeset).unwrap(),
                Some(*NETOLOGY_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn).label, "foxford.ru");
            }
        }
    }

    #[test]
    fn anonymous_cannot_update_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let changeset = build_request("foxford-new.ru");
        let req = shared::build_anonymous_request(&srv, serde_json::to_string(&changeset).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        let conn = get_conn!(pool);
        assert_eq!(find_record(&conn).label, "foxford.ru");
    }
}

mod without_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) {
        let _ = before_each_1(conn);
    }

    #[test]
    fn admin_cannot_update_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let changeset = build_request("foxford-new.ru");
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&changeset).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::NOT_FOUND);
    }

    #[test]
    fn client_cannot_update_alien_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let changeset = build_request("foxford-new.ru");
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&changeset).unwrap(),
            Some(*NETOLOGY_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn anonymous_cannot_update_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let changeset = build_request("foxford-new.ru");
        let req = shared::build_anonymous_request(&srv, serde_json::to_string(&changeset).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

fn build_request(label: &str) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "namespace.update",
        "params": [{
            "id": *FOXFORD_NAMESPACE_ID,
            "data": {
                "label": label
            },
        }],
        "id": "qwerty"
    })
}

fn find_record(conn: &PgConnection) -> Namespace {
    namespace::table
        .find(*FOXFORD_NAMESPACE_ID)
        .get_result(conn)
        .unwrap()
}
