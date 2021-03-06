extern crate abac;
extern crate actix_web;
extern crate chrono;
extern crate diesel;
extern crate env_logger;
extern crate frank_jwt;
#[macro_use]
extern crate iam;
extern crate jsonrpc_core as jsonrpc;
extern crate jsonwebtoken;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate pretty_assertions;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate uuid;

#[macro_use]
mod shared;

mod abac_action_attr;
mod abac_object_attr;
mod abac_policy;
mod abac_subject_attr;
mod account;
mod authn;
mod authz;
mod identity;
mod namespace;
mod ping;
mod rpc;
