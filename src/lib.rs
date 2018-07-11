#![deny(missing_debug_implementations)]

extern crate abac;
extern crate actix;
extern crate actix_web;
extern crate chrono;
extern crate config;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate frank_jwt;
extern crate futures;
extern crate jsonrpc_core as jsonrpc;
#[macro_use]
extern crate jsonrpc_macros;
extern crate jsonwebtoken;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate num_cpus;
extern crate ring;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate uuid;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use actix::prelude::*;
use actix_web::{http, App, HttpResponse};
use diesel::{r2d2, PgConnection};

use actors::DbExecutor;
use rpc::{Meta, Server};

#[macro_use]
pub mod settings;

pub mod abac_attribute;
pub mod actors;
pub mod authn;
pub mod models;
pub mod rpc;
pub mod schema;

lazy_static! {
    static ref SYSTEM_RANDOM: ring::rand::SystemRandom = ring::rand::SystemRandom::new();
}

const TOKEN_TYPE: &str = "Bearer";

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[derive(Debug)]
pub struct AppState {
    pub rpc_server: Server,
    pub rpc_meta: Meta,
}

pub fn build_app(database_url: String) -> App<AppState> {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).unwrap();
    App::with_state(build_app_state(pool))
        .middleware(actix_web::middleware::Logger::default())
        .resource("/", |r| r.method(http::Method::POST).with_async(rpc::index))
        .resource("/auth/{auth_key}/token", |r| {
            use actix_web::pred;

            r.route()
                .filter(pred::Not(
                    pred::Any(pred::Header(
                        "Content-Type",
                        "application/x-www-form-urlencoded",
                    )).or(pred::Header("Content-Type", "application/json")),
                ))
                .f(|_| HttpResponse::NotAcceptable());

            r.method(http::Method::POST)
                .with_async(authn::retrieve::call)
        })
        .resource("/accounts/{key}/refresh", |r| {
            r.method(http::Method::POST)
                .with_async(authn::refresh::call)
        })
        .resource("/accounts/{key}/revoke", |r| {
            r.method(http::Method::POST).with_async(authn::revoke::call)
        })
}

pub fn build_app_state(pool: DbPool) -> AppState {
    let addr = SyncArbiter::start(num_cpus::get(), move || DbExecutor(pool.clone()));

    AppState {
        rpc_server: rpc::build_server(),
        rpc_meta: Meta {
            db: Some(addr.clone()),
            subject: None,
        },
    }
}

pub fn extract_authorization_header(
    headers: &actix_web::http::HeaderMap,
) -> Result<Option<&str>, ()> {
    let auth_header = headers.get("Authorization").map(|v| v.to_str());
    match auth_header {
        Some(Ok(header)) => {
            let mut kv = header.splitn(2, ' ');
            match (kv.next(), kv.next()) {
                (Some(TOKEN_TYPE), Some(v)) => Ok(Some(v)),
                _ => {
                    error!("Bad auth header: {}", header);
                    Err(())
                }
            }
        }
        Some(Err(_)) => {
            error!("Cannot parse auth header");
            Err(())
        }
        None => {
            debug!("Missing auth header");
            Ok(None)
        }
    }
}
