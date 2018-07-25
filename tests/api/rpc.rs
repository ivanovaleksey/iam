use actix_web::{client::ClientRequest, http, test::TestServer, HttpMessage};
use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;

use shared;

lazy_static! {
    static ref ACCOUNT_ID: Uuid = Uuid::new_v4();
}

#[test]
fn with_invalid_auth_header_type() {
    let shared::Server { mut srv, pool: _ } = shared::build_server();

    let access_token = shared::generate_access_token(*ACCOUNT_ID);
    let req = build_request(&srv, &format!("Basic {}", access_token));

    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::UNAUTHORIZED);
}

#[test]
fn with_invalid_access_token_payload() {
    let shared::Server { mut srv, pool: _ } = shared::build_server();

    let access_token = {
        let now = Utc::now().timestamp();
        let token = json!({
            "aud": "iam.netology-group.services".to_owned(),
            "exp": NaiveDateTime::from_timestamp(now + 300, 0).timestamp(),
            "iat": NaiveDateTime::from_timestamp(now, 0).timestamp(),
            "sub": *ACCOUNT_ID,
        });
        shared::sign_access_token(token)
    };
    let req = build_request(&srv, &format!("Bearer {}", access_token));

    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::UNAUTHORIZED);
}

#[test]
fn with_expired_access_token() {
    let shared::Server { mut srv, pool: _ } = shared::build_server();

    let access_token = {
        let now = Utc::now().timestamp();
        let token = json!({
            "aud": "iam.netology-group.services".to_owned(),
            "iss": "foxford.ru".to_owned(),
            "exp": NaiveDateTime::from_timestamp(now - 100, 0).timestamp(),
            "iat": NaiveDateTime::from_timestamp(now - 400, 0).timestamp(),
            "sub": *ACCOUNT_ID,
        });
        shared::sign_access_token(token)
    };
    let req = build_request(&srv, &format!("Bearer {}", access_token));

    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::UNAUTHORIZED);
}

fn build_request(srv: &TestServer, auth_header: &str) -> ClientRequest {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "foo",
        "params": [],
        "id": "qwerty"
    });

    let mut builder = srv.post();
    builder
        .content_type("application/json")
        .header(http::header::AUTHORIZATION, auth_header)
        .json(payload)
        .unwrap()
}
