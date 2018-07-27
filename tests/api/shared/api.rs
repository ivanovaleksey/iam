use actix_web::{self, http, test::TestServer};
use frank_jwt;
use iam;
use serde::ser::Serialize;
use serde_json;
use uuid::Uuid;

use shared::{self, IAM_ACCOUNT_ID};

pub use self::response::{BAD_REQUEST, FORBIDDEN, NOT_FOUND, UNAUTHORIZED};

pub mod request {
    use super::*;

    pub fn build_auth_request(
        srv: &TestServer,
        json: String,
        account_id: Option<Uuid>,
    ) -> actix_web::client::ClientRequest {
        let account_id = account_id.or(Some(*IAM_ACCOUNT_ID));
        build_rpc_request(srv, json, account_id)
    }

    pub fn build_anonymous_request(
        srv: &TestServer,
        json: String,
    ) -> actix_web::client::ClientRequest {
        build_rpc_request(srv, json, None)
    }

    fn build_rpc_request(
        srv: &TestServer,
        json: String,
        account_id: Option<Uuid>,
    ) -> actix_web::client::ClientRequest {
        let mut builder = srv.post();
        builder.content_type("application/json");

        if let Some(account_id) = account_id {
            let auth_header = format!("Bearer {}", generate_iam_access_token(account_id));
            builder.header(http::header::AUTHORIZATION, auth_header);
        }

        builder.body(json).unwrap()
    }
}

mod response {
    use super::*;

    lazy_static! {
        pub static ref BAD_REQUEST: String = {
            let json = r#"{
                "jsonrpc": "2.0",
                "error": {
                    "code": 400,
                    "message": "Bad request"
                },
                "id": "qwerty"
            }"#;
            shared::strip_json(json)
        };
        pub static ref UNAUTHORIZED: String = {
            let json = r#"{
                "jsonrpc": "2.0",
                "error": {
                    "code": 401,
                    "message": "Unauthorized"
                },
                "id": "qwerty"
            }"#;
            shared::strip_json(json)
        };
        pub static ref FORBIDDEN: String = {
            let json = r#"{
                "jsonrpc": "2.0",
                "error": {
                    "code": 403,
                    "message": "Forbidden"
                },
                "id": "qwerty"
            }"#;
            shared::strip_json(json)
        };
        pub static ref NOT_FOUND: String = {
            let json = r#"{
                "jsonrpc": "2.0",
                "error": {
                    "code": 404,
                    "message": "NotFound"
                },
                "id": "qwerty"
            }"#;
            shared::strip_json(json)
        };
    }
}

pub fn generate_iam_access_token(sub: Uuid) -> String {
    let token = iam::authn::jwt::AccessToken::new("foxford.ru".to_owned(), 300, sub);
    sign_iam_access_token(token)
}

pub fn generate_client_access_token(sub: Uuid) -> String {
    let mut token =
        iam::authn::jwt::AccessToken::new("iam.netology-group.services".to_owned(), 300, sub);
    token.iss = "foxford.ru".to_owned();
    sign_client_access_token(token)
}

pub fn generate_refresh_token(refresh_token: &iam::models::RefreshToken) -> String {
    let token =
        iam::authn::jwt::RefreshToken::new("foxford.ru".to_owned(), refresh_token.account_id);
    iam::authn::jwt::RefreshToken::encode(&token, &refresh_token.keys[0]).unwrap()
}

pub fn sign_iam_access_token<T: Serialize>(token: T) -> String {
    let settings = get_settings!();
    frank_jwt::encode(
        json!({}),
        &settings.tokens.key,
        &serde_json::to_value(token).unwrap(),
        frank_jwt::Algorithm::ES256,
    ).unwrap()
}

pub fn sign_client_access_token<T: Serialize>(token: T) -> String {
    use std::{fs::File, io::Read};

    let mut keyfile =
        File::open("tests/keys/foxford/private_key.pem").expect("Missing foxford private key");
    let mut key = String::new();
    keyfile.read_to_string(&mut key).unwrap();

    frank_jwt::encode(
        json!({}),
        &key,
        &serde_json::to_value(token).unwrap(),
        frank_jwt::Algorithm::ES256,
    ).unwrap()
}
