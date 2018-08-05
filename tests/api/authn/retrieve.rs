use actix_web::{client::ClientRequest, test::TestServer, HttpMessage};
use diesel::{self, prelude::*};
use serde::ser::Serialize;
use serde_json;
use uuid::Uuid;

use iam::actors::db;
use iam::authn;
use iam::models::{identity::PrimaryKey, Account, Identity, Namespace, RefreshToken};
use iam::schema::{account, identity, refresh_token};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_NAMESPACE_ID};

lazy_static! {
    static ref FOXFORD_USER_ID: Uuid = Uuid::new_v4();
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ErrorResponse {
    error: String,
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

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

#[test]
fn with_invalid_payload() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let auth_key = authn::AuthKey {
        provider: "foxford.ru".to_owned(),
        label: "oauth2".to_owned(),
    };
    let client_token = shared::generate_client_access_token(*FOXFORD_USER_ID);

    let payload = json!({
        "client_token": client_token,
    });
    let req = build_request(&srv, &auth_key, payload);
    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 400);

    let body = srv.execute(resp.body()).unwrap();
    if let Ok(resp) = serde_json::from_slice::<ErrorResponse>(&body) {
        assert_eq!(resp.error, "invalid_request");
    } else {
        panic!("{:?}", body);
    }
}

#[test]
fn with_invalid_grant_type() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let auth_key = authn::AuthKey {
        provider: "foxford.ru".to_owned(),
        label: "oauth2".to_owned(),
    };
    let client_token = shared::generate_client_access_token(*FOXFORD_USER_ID);

    let payload = json!({
        "grant_type": "authorization_code",
        "client_token": client_token,
    });
    let req = build_request(&srv, &auth_key, payload);
    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 400);

    let body = srv.execute(resp.body()).unwrap();
    if let Ok(resp) = serde_json::from_slice::<ErrorResponse>(&body) {
        assert_eq!(resp.error, "invalid_request");
    } else {
        panic!("{:?}", body);
    }
}

#[test]
fn with_invalid_expires_in() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let auth_key = authn::AuthKey {
        provider: "foxford.ru".to_owned(),
        label: "oauth2".to_owned(),
    };
    let client_token = shared::generate_client_access_token(*FOXFORD_USER_ID);

    let payload = json!({
        "grant_type": "client_credentials",
        "client_token": client_token,
        "expires_in": 14401
    });
    let req = build_request(&srv, &auth_key, payload);
    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 400);

    let body = srv.execute(resp.body()).unwrap();
    if let Ok(resp) = serde_json::from_slice::<ErrorResponse>(&body) {
        assert_eq!(resp.error, "invalid_request");
    } else {
        panic!("{:?}", body);
    }
}

#[test]
fn with_invalid_client_token_payload() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let auth_key = authn::AuthKey {
        provider: "foxford.ru".to_owned(),
        label: "oauth2".to_owned(),
    };
    let client_token = {
        use chrono::{NaiveDateTime, Utc};

        let now = Utc::now().timestamp();
        let token = json!({
            "aud": "iam.netology-group.services".to_owned(),
            "exp": NaiveDateTime::from_timestamp(now + 300, 0).timestamp(),
            "iat": NaiveDateTime::from_timestamp(now, 0).timestamp(),
            "sub": *FOXFORD_USER_ID,
        });
        shared::sign_client_access_token(token)
    };

    let payload = json!({
        "grant_type": "client_credentials",
        "client_token": client_token,
    });
    let req = build_request(&srv, &auth_key, payload);
    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 400);

    let body = srv.execute(resp.body()).unwrap();
    if let Ok(resp) = serde_json::from_slice::<ErrorResponse>(&body) {
        assert_eq!(resp.error, "invalid_client");
    } else {
        panic!("{:?}", body);
    }
}

#[test]
fn with_expired_client_token() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let auth_key = authn::AuthKey {
        provider: "foxford.ru".to_owned(),
        label: "oauth2".to_owned(),
    };
    let client_token = {
        use chrono::{NaiveDateTime, Utc};

        let now = Utc::now().timestamp();
        let token = json!({
            "aud": "iam.netology-group.services".to_owned(),
            "iss": "foxford.ru".to_owned(),
            "exp": NaiveDateTime::from_timestamp(now - 100, 0).timestamp(),
            "iat": NaiveDateTime::from_timestamp(now - 400, 0).timestamp(),
            "sub": *FOXFORD_USER_ID,
        });
        shared::sign_client_access_token(token)
    };

    let payload = json!({
        "grant_type": "client_credentials",
        "client_token": client_token,
    });
    let req = build_request(&srv, &auth_key, payload);
    let resp = srv.execute(req.send()).unwrap();
    assert_eq!(resp.status(), 400);

    let body = srv.execute(resp.body()).unwrap();
    if let Ok(resp) = serde_json::from_slice::<ErrorResponse>(&body) {
        assert_eq!(resp.error, "invalid_client");
    } else {
        panic!("{:?}", body);
    }
}

mod with_existing_identity {
    use super::*;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> (Identity, Account, RefreshToken) {
        let _ = before_each_1(conn);

        let pk = PrimaryKey {
            provider: *FOXFORD_NAMESPACE_ID,
            label: "oauth2".to_owned(),
            uid: FOXFORD_USER_ID.to_string(),
        };
        db::identity::insert::insert_identity_with_account(conn, pk).unwrap()
    }

    #[test]
    fn with_enabled_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        let (account, created_refresh_token) = {
            let conn = get_conn!(pool);
            let (_, account, token) = before_each_2(&conn);
            (account, token)
        };

        let auth_key = authn::AuthKey {
            provider: "foxford.ru".to_owned(),
            label: "oauth2".to_owned(),
        };
        let client_token = shared::generate_client_access_token(*FOXFORD_USER_ID);

        let payload = json!({
            "grant_type": "client_credentials",
            "client_token": client_token,
        });
        let req = build_request(&srv, &auth_key, payload);
        let resp = srv.execute(req.send()).unwrap();
        assert_eq!(resp.status(), 200);

        let body = srv.execute(resp.body()).unwrap();
        if let Ok(resp) = serde_json::from_slice::<authn::retrieve::Response>(&body) {
            let raw_token = authn::jwt::RawToken {
                kind: authn::jwt::RawTokenKind::Iam,
                value: &resp.access_token,
            };
            let access_token = authn::jwt::AccessToken::decode(&raw_token).unwrap();

            assert_eq!(account.id, access_token.sub);

            let refresh_token = authn::jwt::RefreshToken::decode(
                &resp.refresh_token,
                &created_refresh_token.keys[0],
            );

            assert!(refresh_token.is_ok());
        } else {
            panic!("{:?}", body);
        }
    }

    #[test]
    fn with_disabled_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let (_, account, _) = before_each_2(&conn);

            diesel::update(account::table.find(account.id))
                .set(account::disabled_at.eq(diesel::dsl::now))
                .execute(&conn)
                .unwrap();
        }

        let auth_key = authn::AuthKey {
            provider: "foxford.ru".to_owned(),
            label: "oauth2".to_owned(),
        };
        let client_token = shared::generate_client_access_token(*FOXFORD_USER_ID);

        let payload = json!({
            "grant_type": "client_credentials",
            "client_token": client_token,
        });
        let req = build_request(&srv, &auth_key, payload);
        let resp = srv.execute(req.send()).unwrap();
        assert_eq!(resp.status(), 403);
    }
}

mod without_existing_identity {
    use super::*;

    #[test]
    fn test() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let auth_key = authn::AuthKey {
            provider: "foxford.ru".to_owned(),
            label: "oauth2".to_owned(),
        };
        let client_token = shared::generate_client_access_token(*FOXFORD_USER_ID);

        let payload = json!({
            "grant_type": "client_credentials",
            "client_token": client_token,
        });
        let req = build_request(&srv, &auth_key, payload);
        let resp = srv.execute(req.send()).unwrap();
        assert_eq!(resp.status(), 200);

        let body = srv.execute(resp.body()).unwrap();
        if let Ok(resp) = serde_json::from_slice::<authn::retrieve::Response>(&body) {
            let raw_token = authn::jwt::RawToken {
                kind: authn::jwt::RawTokenKind::Iam,
                value: &resp.access_token,
            };
            let access_token = authn::jwt::AccessToken::decode(&raw_token).unwrap();
            let account_id = access_token.sub;

            {
                let conn = get_conn!(pool);

                let pk = PrimaryKey {
                    provider: *FOXFORD_NAMESPACE_ID,
                    label: "oauth2".to_owned(),
                    uid: FOXFORD_USER_ID.to_string(),
                };
                let identity = identity::table
                    .find(pk.as_tuple())
                    .get_result::<Identity>(&conn)
                    .unwrap();

                assert_eq!(identity.account_id, account_id);

                let created_refresh_token = refresh_token::table
                    .find(account_id)
                    .get_result::<RefreshToken>(&conn)
                    .unwrap();

                let refresh_token = authn::jwt::RefreshToken::decode(
                    &resp.refresh_token,
                    &created_refresh_token.keys[0],
                ).unwrap();

                assert_eq!(refresh_token.sub, account_id);
            }
        } else {
            panic!("{:?}", body);
        }
    }
}

fn build_request<T: Serialize>(
    srv: &TestServer,
    auth_key: &authn::AuthKey,
    payload: T,
) -> ClientRequest {
    use actix_web::http::Method;

    let url = format!("/auth/{}/token", auth_key);
    let mut builder = srv.client(Method::POST, &url);
    builder
        .content_type("application/json")
        .json(payload)
        .unwrap()
}
