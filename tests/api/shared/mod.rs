use actix_web::{self, http, test::TestServer};
use diesel;
use iam;
use uuid::Uuid;

pub mod api;
pub mod db;

lazy_static! {
    pub static ref IAM_ACCOUNT_ID: Uuid =
        Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap();
    pub static ref IAM_NAMESPACE_ID: Uuid =
        Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap();
    pub static ref FOXFORD_ACCOUNT_ID: Uuid = Uuid::new_v4();
    pub static ref FOXFORD_NAMESPACE_ID: Uuid = Uuid::new_v4();
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
            app.resource("/", |r| r.post().h(iam::call));
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
        use iam::settings::SETTINGS;
        use jwt;

        let header = json!({});
        let payload = json!({
            "sub": account_id,
        });
        let settings = SETTINGS.read().unwrap();
        let token = jwt::encode(
            header,
            &settings.private_key,
            &payload,
            jwt::Algorithm::ES256,
        ).unwrap();
        let auth_header = format!("Bearer {}", token);
        builder.header(http::header::AUTHORIZATION, auth_header);
    }

    builder.body(json).unwrap()
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
