use diesel::{self, prelude::*};
use serde_json;

use iam::models::{Account, Namespace};
use iam::schema::namespace;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, IAM_NAMESPACE_ID};

lazy_static! {
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "account_id": "FOXFORD_ACCOUNT_ID",
                "created_at": "2018-05-30T08:40:00",
                "enabled": false,
                "id": "FOXFORD_NAMESPACE_ID",
                "label": "foxford.ru"
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("FOXFORD_ACCOUNT_ID", &FOXFORD_ACCOUNT_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

        shared::strip_json(&json)
    };
}

#[must_use]
fn before_each_1(conn: &PgConnection) -> (Account, Namespace) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    (iam_account, iam_namespace)
}

mod with_enabled_namespace {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> Namespace {
        let _ = before_each_1(conn);

        let foxford_account = create_account(conn, AccountKind::Foxford);
        let foxford_namespace = create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

        foxford_namespace
    }

    #[test]
    fn admin_can_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *EXPECTED);

        {
            let conn = get_conn!(pool);
            assert!(!find_record(&conn).enabled);

            assert_eq!(namespace_objects_count(&conn), Ok(0));
        }
    }

    #[test]
    fn client_can_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *EXPECTED);

        {
            let conn = get_conn!(pool);
            assert!(!find_record(&conn).enabled);

            assert_eq!(namespace_objects_count(&conn), Ok(0));
        }
    }

    #[test]
    fn anonymous_cannot_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        {
            let conn = get_conn!(pool);
            assert!(find_record(&conn).enabled);

            assert_eq!(namespace_objects_count(&conn), Ok(2));
        }
    }
}

mod with_disabled_namespace {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> Namespace {
        let _ = before_each_1(conn);

        let foxford_account = create_account(conn, AccountKind::Foxford);
        let foxford_namespace = create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

        diesel::update(&foxford_namespace)
            .set(namespace::enabled.eq(false))
            .execute(conn)
            .unwrap();

        foxford_namespace
    }

    #[test]
    fn admin_cannot_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::NOT_FOUND);

        {
            let conn = get_conn!(pool);
            assert!(!find_record(&conn).enabled);
        }
    }

    #[test]
    fn client_cannot_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        {
            let conn = get_conn!(pool);
            assert!(!find_record(&conn).enabled);
        }
    }

    #[test]
    fn anonymous_cannot_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        {
            let conn = get_conn!(pool);
            assert!(!find_record(&conn).enabled);
        }
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
    fn admin_cannot_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::NOT_FOUND);
    }

    #[test]
    fn client_cannot_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn anonymous_cannot_delete_namespace() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

fn build_request() -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "namespace.delete",
        "params": [{
            "id": *FOXFORD_NAMESPACE_ID
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

fn namespace_objects_count(conn: &PgConnection) -> diesel::QueryResult<usize> {
    use abac::schema::abac_object;
    use abac::types::AbacAttribute;
    use iam::abac_attribute::UriKind;

    abac_object::table
        .filter(abac_object::inbound.eq(AbacAttribute::new(
            *IAM_NAMESPACE_ID,
            UriKind::Namespace(*FOXFORD_NAMESPACE_ID),
        )))
        .execute(conn)
}
