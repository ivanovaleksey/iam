extern crate actix;
extern crate actix_web;
extern crate bytes;
extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate jsonrpc_core as jsonrpc;
#[macro_use]
extern crate jsonrpc_macros;
#[macro_use]
extern crate log;
extern crate num_cpus;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

use actix::prelude::*;
use actix_web::{App, AsyncResponder, FutureResponse, HttpMessage, HttpRequest, HttpResponse};
use bytes::Bytes;
use diesel::{r2d2, PgConnection};
use futures::Future;

use actors::DbExecutor;
use rpc::{Meta, Server};

pub mod actors;
pub mod models;
pub mod rpc;
pub mod schema;

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub struct AppState {
    pub rpc_server: Server,
    pub rpc_meta: Meta,
}

pub fn build_app(database_url: String) -> App<AppState> {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::new(manager).unwrap();
    App::with_state(build_app_state(pool)).resource("/", |r| r.h(call))
}

pub fn build_app_state(pool: DbPool) -> AppState {
    let addr = SyncArbiter::start(num_cpus::get(), move || DbExecutor(pool.clone()));

    AppState {
        rpc_server: rpc::build_server(),
        rpc_meta: Meta {
            db: Some(addr.clone()),
        },
    }
}

pub fn call(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let meta = req.state().rpc_meta.clone();

    req.clone()
        .body()
        .from_err()
        .and_then(move |bytes: Bytes| {
            let bytes = bytes.to_vec();
            // TODO: do not unwrap
            let msg = String::from_utf8(bytes).unwrap();

            req.state()
                .rpc_server
                .handle_request(&msg, meta)
                .map_err(actix_web::error::ErrorInternalServerError)
                .and_then(|resp| {
                    if let Some(resp) = resp {
                        Ok(HttpResponse::Ok().body(resp))
                    } else {
                        Ok(HttpResponse::Ok().into())
                    }
                })
        })
        .responder()
}
