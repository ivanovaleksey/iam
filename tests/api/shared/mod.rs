extern crate actix_web;
extern crate diesel;
extern crate iam;

use actix_web::test::TestServer;
use iam::DbPool;

pub struct Server {
    pub srv: TestServer,
    pub pool: DbPool,
}

pub fn build_server() -> Server {
    use std::env;

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(database_url);

    let pool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to build pool");

    let pool1 = pool.clone();
    let srv =
        TestServer::build_with_state(move || iam::build_app_state(pool1.clone())).start(|app| {
            app.resource("/", |r| r.h(iam::call));
        });

    Server { srv, pool }
}

pub fn strip_json(json: &str) -> String {
    json.replace('\n', "").replace(' ', "")
}
