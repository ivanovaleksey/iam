use futures::future::{self, Future};
use jsonrpc;

use actors::db::{abac_policy, authz::Authz};
use rpc;

pub type Request = rpc::abac_policy::read::Request;
pub type Response = rpc::abac_policy::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.namespace_id;
            move |subject_id| {
                let msg = Authz::execute_namespace_message(namespace_id, subject_id);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_policy::delete::Delete::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("abac policy delete res: {:?}", res);
                        Ok(Response::from(res?))
                    })
            }
        })
}
