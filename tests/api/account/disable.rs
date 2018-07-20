use diesel::{self, prelude::*};
use jsonrpc;
use serde_json;
use uuid::Uuid;

use iam::models::{Account, Namespace};
use iam::schema::account;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, IAM_ACCOUNT_ID};

lazy_static! {
    static ref USER_ACCOUNT_ID_1: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID_2: Uuid = Uuid::new_v4();
}

#[must_use]
fn before_each_1(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    let foxford_account = create_account(conn, AccountKind::Foxford);
    let foxford_namespace = create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

    let netology_account = create_account(conn, AccountKind::Netology);
    let _netology_namespace = create_namespace(conn, NamespaceKind::Netology(netology_account.id));

    let _user_account_2 = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_2));

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_enabled_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> Account {
        let _ = before_each_1(conn);

        create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_1))
    }

    #[test]
    fn admin_can_disable_account() {
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

        if let Ok(_) = serde_json::from_slice::<jsonrpc::Success>(&body) {
            {
                let conn = get_conn!(pool);
                let record = find_record(&conn);
                assert!(record.disabled_at.is_some());
            }
        } else {
            panic!("{:?}", body);
        }
    }

    #[test]
    fn client_cannot_disable_account() {
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
            let record = find_record(&conn);
            assert!(record.disabled_at.is_none());
        }
    }

    #[test]
    fn user_cannot_disable_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        {
            let conn = get_conn!(pool);
            let record = find_record(&conn);
            assert!(record.disabled_at.is_none());
        }
    }

    #[test]
    fn anonymous_cannot_disable_account() {
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
            let record = find_record(&conn);
            assert!(record.disabled_at.is_none());
        }
    }
}

mod with_disabled_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> Account {
        let _ = before_each_1(conn);

        let account = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_1));

        diesel::update(&account)
            .set(account::disabled_at.eq(diesel::dsl::now))
            .execute(conn)
            .unwrap();

        account
    }

    #[test]
    fn admin_cannot_disable_account() {
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
    fn client_cannot_disable_account() {
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
    fn user_cannot_disable_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn anonymous_cannot_disable_account() {
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

mod with_deleted_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> Account {
        let _ = before_each_1(conn);

        let account = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_1));

        diesel::update(&account)
            .set(account::deleted_at.eq(diesel::dsl::now))
            .execute(conn)
            .unwrap();

        account
    }

    #[test]
    fn admin_cannot_disable_account() {
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
            let record = find_record(&conn);
            assert!(record.disabled_at.is_none());
        }
    }

    #[test]
    fn client_cannot_disable_account() {
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
            let record = find_record(&conn);
            assert!(record.disabled_at.is_none());
        }
    }

    #[test]
    fn user_cannot_disable_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*USER_ACCOUNT_ID_2),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        {
            let conn = get_conn!(pool);
            let record = find_record(&conn);
            assert!(record.disabled_at.is_none());
        }
    }

    #[test]
    fn anonymous_cannot_disable_account() {
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
            let record = find_record(&conn);
            assert!(record.disabled_at.is_none());
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
    fn admin_cannot_disable_account() {
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
    fn client_cannot_disable_account() {
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
    fn user_cannot_disable_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*USER_ACCOUNT_ID_2),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn anonymous_cannot_disable_account() {
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
        "method": "account.disable",
        "params": [{
            "id": *USER_ACCOUNT_ID_1
        }],
        "id": "qwerty"
    })
}

fn find_record(conn: &PgConnection) -> Account {
    account::table
        .find(*USER_ACCOUNT_ID_1)
        .get_result(conn)
        .unwrap()
}
