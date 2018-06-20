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
extern crate frank_jwt as jwt;
extern crate futures;
extern crate jsonrpc_core as jsonrpc;
#[macro_use]
extern crate jsonrpc_macros;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate num_cpus;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

use actix::prelude::*;
use actix_web::{App, AsyncResponder, FutureResponse, HttpMessage, HttpRequest, HttpResponse};
use diesel::{r2d2, PgConnection};
use futures::future::{self, Either, Future};

use actors::DbExecutor;
use rpc::{Meta, Server};

pub mod actors;
pub mod models;
pub mod rpc;
pub mod schema;
pub mod settings;

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
        .resource("/", |r| r.post().h(call))
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

#[derive(Debug, Deserialize)]
struct JwtPayload {
    sub: uuid::Uuid,
}

pub fn call(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let mut meta = req.state().rpc_meta.clone();

    req.clone()
        .json()
        .from_err()
        .and_then(move |request: jsonrpc::Request| {
            let auth_header = req.headers().get("Authorization").map(|v| v.to_str());

            let res = match auth_header {
                Some(Ok(header)) => {
                    let mut kv = header.splitn(2, ' ');
                    match (kv.next(), kv.next()) {
                        (Some("Bearer"), Some(v)) => {
                            let settings = settings::SETTINGS.read().unwrap();
                            if let Ok((_header, payload)) = jwt::decode(
                                &v.to_owned(),
                                &settings.public_key,
                                jwt::Algorithm::ES256,
                            ) {
                                match serde_json::from_value::<JwtPayload>(payload) {
                                    Ok(payload) => {
                                        debug!("JWT payload: {:?}", payload);
                                        meta.subject = Some(payload.sub);
                                        Ok(())
                                    }
                                    Err(_) => {
                                        debug!("Bad JWT");
                                        Err(())
                                    }
                                }
                            } else {
                                debug!("Bad signature");
                                Err(())
                            }
                        }
                        _ => {
                            debug!("Bad auth header");
                            Err(())
                        }
                    }
                }
                Some(Err(_)) => {
                    debug!("Cannot parse auth header");
                    Err(())
                }
                None => {
                    debug!("Missing auth header");
                    Ok(())
                }
            };

            if res.is_ok() {
                Either::A(
                    req.state()
                        .rpc_server
                        .handle_rpc_request(request, meta)
                        .map_err(internal_error),
                )
            } else {
                Either::B(reject_request(&request).map_err(internal_error))
            }
        })
        .then(|res| {
            res.or_else(|_| {
                let e = jsonrpc::Error::new(jsonrpc::ErrorCode::ParseError);
                let resp = jsonrpc::Response::from(e, Some(jsonrpc::Version::V2));
                Ok(Some(resp))
            })
        })
        .and_then(|resp| {
            if let Some(resp) = resp {
                let resp_str = serde_json::to_string(&resp).unwrap();
                Ok(HttpResponse::Ok().body(resp_str))
            } else {
                Ok(HttpResponse::Ok().into())
            }
        })
        .responder()
}

fn internal_error(_e: ()) -> actix_web::Error {
    actix_web::error::ErrorInternalServerError("")
}

fn reject_request(
    request: &jsonrpc::Request,
) -> impl Future<Item = Option<jsonrpc::Response>, Error = ()> {
    match request {
        jsonrpc::Request::Single(call) => {
            let output = reject_call(call);
            let res = output.map(|o| o.map(jsonrpc::Response::Single));
            Either::A(res)
        }
        jsonrpc::Request::Batch(calls) => {
            let futures: Vec<_> = calls.iter().map(|c| reject_call(c)).collect();
            let res = future::join_all(futures).map(|outs| {
                let outs: Vec<_> = outs.into_iter().filter_map(|v| v).collect();
                Some(jsonrpc::Response::Batch(outs))
            });
            Either::B(res)
        }
    }
}

fn reject_call(call: &jsonrpc::Call) -> impl Future<Item = Option<jsonrpc::Output>, Error = ()> {
    let err = jsonrpc::Error {
        code: jsonrpc::ErrorCode::ServerError(401),
        message: "Unauthorized".to_owned(),
        data: None,
    };

    let output = match call {
        jsonrpc::Call::MethodCall(method) => {
            jsonrpc::Output::from(Err(err), method.id.clone(), method.jsonrpc)
        }
        jsonrpc::Call::Notification(notification) => {
            jsonrpc::Output::from(Err(err), jsonrpc::Id::Null, notification.jsonrpc)
        }
        jsonrpc::Call::Invalid(_id) => jsonrpc::Output::from(Err(err), jsonrpc::Id::Null, None),
    };

    future::ok(Some(output))
}
