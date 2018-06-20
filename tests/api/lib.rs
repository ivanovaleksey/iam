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

//mod abac_action_attr;
//mod abac_object_attr;
//mod abac_policy;
//mod abac_subject_attr;
mod account;
mod authz;
mod identity;
mod namespace;
mod ping;

mod shared;
