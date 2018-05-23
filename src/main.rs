extern crate actix;
extern crate actix_web;
extern crate diesel;
extern crate env_logger;
extern crate iam;

use actix::prelude::*;
use actix_web::server;

use std::env;

fn main() {
    env_logger::init();
    iam::settings::init().expect("Failed to initialize settings");

    let database_url = env::var("DATABASE_URL").unwrap();

    let sys = System::new("iam");

    let app = move || iam::build_app(database_url.clone());
    server::new(app).bind("127.0.0.1:8080").unwrap().start();

    let _ = sys.run();
}
