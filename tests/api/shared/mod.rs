use actix_web::{self, http, test::TestServer};
use diesel;
use iam;
use serde::ser::Serialize;
use serde_json;
use uuid::Uuid;

pub mod api;
pub mod db;

lazy_static! {
    pub static ref IAM_ACCOUNT_ID: Uuid = Uuid::new_v4();
    pub static ref IAM_NAMESPACE_ID: Uuid =
        Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap();
    pub static ref FOXFORD_ACCOUNT_ID: Uuid = Uuid::new_v4();
    pub static ref FOXFORD_NAMESPACE_ID: Uuid = Uuid::new_v4();
    pub static ref NETOLOGY_ACCOUNT_ID: Uuid = Uuid::new_v4();
    pub static ref NETOLOGY_NAMESPACE_ID: Uuid = Uuid::new_v4();
}

#[macro_export]
macro_rules! get_conn {
    ($pool:ident) => {
        $pool.get().expect("Failed to get connection from pool")
    };
}

pub struct Server {
    pub srv: TestServer,
    pub pool: iam::DbPool,
}

pub fn build_server() -> Server {
    use std::env;

    init();

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(database_url);

    let pool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to build pool");

    let pool1 = pool.clone();
    let srv =
        TestServer::build_with_state(move || iam::build_app_state(pool1.clone())).start(|app| {
            app.resource("/", |r| {
                r.method(http::Method::POST).with_async(iam::rpc::index)
            }).resource("/auth/{auth_key}/token", |r| {
                    use actix_web::{pred, HttpResponse};

                    r.route()
                        .filter(pred::Not(
                            pred::Any(pred::Header(
                                "Content-Type",
                                "application/x-www-form-urlencoded",
                            )).or(pred::Header("Content-Type", "application/json")),
                        ))
                        .f(|_| HttpResponse::NotAcceptable());

                    r.method(http::Method::POST)
                        .with_async(iam::authn::retrieve::call)
                })
                .resource("/accounts/{key}/refresh", |r| {
                    r.method(http::Method::POST)
                        .with_async(iam::authn::refresh::call)
                })
                .resource("/accounts/{key}/revoke", |r| {
                    r.method(http::Method::POST)
                        .with_async(iam::authn::revoke::call)
                });
        });

    Server { srv, pool }
}

pub fn build_auth_request(
    srv: &TestServer,
    json: String,
    account_id: Option<Uuid>,
) -> actix_web::client::ClientRequest {
    let account_id = account_id.or(Some(*IAM_ACCOUNT_ID));
    build_rpc_request(srv, json, account_id)
}

pub fn build_anonymous_request(srv: &TestServer, json: String) -> actix_web::client::ClientRequest {
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
        let auth_header = format!("Bearer {}", generate_access_token(account_id));
        builder.header(http::header::AUTHORIZATION, auth_header);
    }

    builder.body(json).unwrap()
}

pub fn generate_access_token(sub: Uuid) -> String {
    let token = iam::authn::jwt::AccessToken::new("foxford.ru".to_owned(), 300, sub);
    sign_access_token(token)
}

pub fn generate_refresh_token(refresh_token: &iam::models::RefreshToken) -> String {
    let token =
        iam::authn::jwt::RefreshToken::new("foxford.ru".to_owned(), refresh_token.account_id);
    iam::authn::jwt::RefreshToken::encode(&token, &refresh_token.keys[0]).unwrap()
}

pub fn sign_access_token<T: Serialize>(token: T) -> String {
    use frank_jwt;

    let settings = get_settings!();
    frank_jwt::encode(
        json!({}),
        &settings.tokens.key,
        &serde_json::to_value(token).unwrap(),
        frank_jwt::Algorithm::ES256,
    ).unwrap()
}

fn init() {
    use env_logger;

    let _ = env_logger::try_init();
    iam::settings::init().expect("Failed to initialize settings");
}

pub fn strip_json(json: &str) -> String {
    json.replace('\n', "")
        .replace("  ", "")
        .replace("\": ", "\":")
}
