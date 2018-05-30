use actix_web::{self, http, test::TestServer};
use diesel;
use iam;

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

pub fn build_rpc_request(srv: &TestServer, json: String) -> actix_web::client::ClientRequest {
    srv.post()
        .header(http::header::AUTHORIZATION, "Bearer eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIyNWEwYzM2Ny03NTZhLTQyZTEtYWM1YS1lN2EyYjZiNjQ0MjAifQ==.MEYCIQDEUAevIcmf-MK7dPZpUPoPxemOTKZZeUYC7NbGDsI-9gIhALfQKFCoc761wS7CcIy0nDa54-QhiIAGeW1ObWv_GxDz")
        .content_type("application/json")
        .body(json)
        .unwrap()
}

pub fn build_anonymous_request(srv: &TestServer, json: String) -> actix_web::client::ClientRequest {
    srv.post()
        .content_type("application/json")
        .body(json)
        .unwrap()
}

fn init() {
    use env_logger;

    let _ = env_logger::try_init();
    iam::settings::init().expect("Failed to initialize settings");
}

pub fn strip_json(json: &str) -> String {
    json.replace('\n', "").replace(' ', "")
}
