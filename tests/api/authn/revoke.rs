use actix_web::{client::ClientRequest, http, test::TestServer, HttpMessage};
use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use iam::actors::db;
use iam::authn;
use iam::models::{identity::PrimaryKey, RefreshToken};
use iam::schema::{account, refresh_token};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_NAMESPACE_ID};

lazy_static! {
    static ref FOXFORD_USER_ID: Uuid = Uuid::new_v4();
}

#[must_use]
fn before_each_1(conn: &PgConnection) -> RefreshToken {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    let foxford_account = create_account(conn, AccountKind::Foxford);
    let _foxford_namespace = create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

    let pk = PrimaryKey {
        provider: *FOXFORD_NAMESPACE_ID,
        label: "oauth2".to_owned(),
        uid: FOXFORD_USER_ID.to_string(),
    };
    let (_, _, refresh_token) =
        db::identity::insert::insert_identity_with_account(&conn, pk).unwrap();

    refresh_token
}

#[test]
fn without_authorization() {
    let shared::Server { mut srv, pool } = shared::build_server();

    let refresh_token = {
        let conn = get_conn!(pool);
        before_each_1(&conn)
    };

    let req = {
        use actix_web::http::Method;

        let url = format!("/accounts/{}/revoke", refresh_token.account_id.to_string());
        let mut builder = srv.client(Method::POST, &url);

        builder.content_type("application/json").finish().unwrap()
    };

    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 403);
}

#[test]
fn with_invalid_signature() {
    let shared::Server { mut srv, pool } = shared::build_server();

    let refresh_token = {
        let conn = get_conn!(pool);
        before_each_1(&conn)
    };

    let mut token = shared::generate_refresh_token(&refresh_token);
    token.push_str("qwerty");

    let req = build_request(&srv, &refresh_token.account_id.to_string(), &token);

    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 401);
}

#[test]
fn with_enabled_account() {
    let shared::Server { mut srv, pool } = shared::build_server();

    let old_refresh_token = {
        let conn = get_conn!(pool);
        before_each_1(&conn)
    };

    let token = shared::generate_refresh_token(&old_refresh_token);
    let req = build_request(&srv, &old_refresh_token.account_id.to_string(), &token);

    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 200);

    let body = srv.execute(resp.body()).unwrap();
    if let Ok(resp) = serde_json::from_slice::<authn::revoke::Response>(&body) {
        let old_key = &old_refresh_token.keys[0];
        assert!(authn::jwt::RefreshToken::decode(&resp.refresh_token, old_key).is_err());

        let new_refresh_token = {
            let conn = get_conn!(pool);
            refresh_token::table
                .find(old_refresh_token.account_id)
                .get_result::<RefreshToken>(&conn)
                .unwrap()
        };
        let new_key = &new_refresh_token.keys[0];
        let refresh_token = authn::jwt::RefreshToken::decode(&resp.refresh_token, new_key).unwrap();
        assert_eq!(refresh_token.sub, old_refresh_token.account_id);
    } else {
        panic!("{:?}", body);
    }
}

#[test]
fn with_disabled_account() {
    let shared::Server { mut srv, pool } = shared::build_server();

    let refresh_token = {
        let conn = get_conn!(pool);
        let token = before_each_1(&conn);

        diesel::update(account::table.find(token.account_id))
            .set(account::disabled_at.eq(diesel::dsl::now))
            .execute(&conn)
            .unwrap();

        token
    };

    let token = shared::generate_refresh_token(&refresh_token);
    let req = build_request(&srv, &refresh_token.account_id.to_string(), &token);

    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 403);
}

#[test]
fn without_existing_account() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let account_id = Uuid::new_v4();
    let req = build_request(&srv, &account_id.to_string(), "qwerty");

    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 404);
}

mod with_me {
    use super::*;

    #[test]
    fn with_valid_signature() {
        let shared::Server { mut srv, pool } = shared::build_server();

        let old_refresh_token = {
            let conn = get_conn!(pool);
            before_each_1(&conn)
        };

        let token = shared::generate_refresh_token(&old_refresh_token);
        let req = build_request(&srv, "me", &token);

        let resp = srv.execute(req.send()).unwrap();
        assert_eq!(resp.status(), 200);

        let body = srv.execute(resp.body()).unwrap();
        if let Ok(resp) = serde_json::from_slice::<authn::revoke::Response>(&body) {
            let old_key = &old_refresh_token.keys[0];
            assert!(authn::jwt::RefreshToken::decode(&resp.refresh_token, old_key).is_err());

            let new_refresh_token = {
                let conn = get_conn!(pool);
                refresh_token::table
                    .find(old_refresh_token.account_id)
                    .get_result::<RefreshToken>(&conn)
                    .unwrap()
            };
            let new_key = &new_refresh_token.keys[0];
            let refresh_token =
                authn::jwt::RefreshToken::decode(&resp.refresh_token, new_key).unwrap();
            assert_eq!(refresh_token.sub, old_refresh_token.account_id);
        } else {
            panic!("{:?}", body);
        }
    }

    #[test]
    fn with_invalid_signature() {
        let shared::Server { mut srv, pool } = shared::build_server();

        let refresh_token = {
            let conn = get_conn!(pool);
            before_each_1(&conn)
        };

        let mut token = shared::generate_refresh_token(&refresh_token);
        token.push_str("qwerty");

        let req = build_request(&srv, "me", &token);

        let resp = srv.execute(req.send()).unwrap();
        assert_eq!(resp.status(), 401);
    }
}

#[test]
fn when_token_without_key() {
    let shared::Server { mut srv, pool } = shared::build_server();

    let refresh_token = {
        let conn = get_conn!(pool);
        let token = before_each_1(&conn);

        diesel::update(refresh_token::table.find(token.account_id))
            .set(refresh_token::keys.eq(Vec::<Vec<u8>>::new()))
            .execute(&conn)
            .unwrap();

        token
    };

    let token = shared::generate_refresh_token(&refresh_token);
    let req = build_request(&srv, &refresh_token.account_id.to_string(), &token);

    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 500);
}

fn build_request(srv: &TestServer, key: &str, token: &str) -> ClientRequest {
    use actix_web::http::Method;

    let url = format!("/accounts/{}/revoke", key);
    let mut builder = srv.client(Method::POST, &url);

    builder
        .content_type("application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .finish()
        .unwrap()
}
