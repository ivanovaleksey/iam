extern crate actix;
extern crate actix_web;
extern crate diesel;
extern crate env_logger;
extern crate iam;
extern crate migrations_internals;
#[macro_use]
extern crate log;

use actix::prelude::*;
use actix_web::server;
use diesel::{r2d2, PgConnection};

use std::env;

fn main() {
    env_logger::init();
    iam::settings::init().expect("Failed to initialize settings");

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).unwrap();

    {
        let conn = pool.get().expect("Failed to get a connection from pool");
        match migrations_internals::any_pending_migrations(&conn) {
            Ok(false) => {}
            Ok(true) => {
                error!("There are pending migrations");
                std::process::exit(1);
            }
            Err(e) => {
                error!("{}", e);
                std::process::exit(1);
            }
        }
    }

    let sys = System::new("iam");

    let app = move || iam::build_app(pool.clone());
    server::new(app).bind("0.0.0.0:8080").unwrap().start();

    let _ = sys.run();
}
