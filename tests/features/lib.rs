extern crate abac;
extern crate actix_web;
extern crate chrono;
extern crate diesel;
extern crate env_logger;
extern crate frank_jwt as jwt;
extern crate iam;
extern crate jsonrpc_core as jsonrpc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pretty_assertions;
#[macro_use]
extern crate serde_json;
extern crate uuid;

#[macro_use]
mod shared;

mod client_creates_abac_object;
mod client_removes_admin_ability_on_abac_object;
