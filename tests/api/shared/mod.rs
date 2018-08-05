use actix_web::{http, test::TestServer};
use diesel;
use iam;
use uuid::Uuid;

pub use shared::api::request::{build_anonymous_request, build_auth_request};
pub use shared::api::{
    generate_client_access_token, generate_iam_access_token, generate_refresh_token,
    sign_client_access_token, sign_iam_access_token,
};

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
