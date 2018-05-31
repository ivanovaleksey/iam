use futures::future::{self, Future};
use jsonrpc;

use actors::db::{abac_object_attr, authz::Authz};
use rpc;

pub type Request = rpc::abac_object_attr::create::Request;
pub type Response = rpc::abac_object_attr::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = meta.subject.ok_or(rpc::error::Error::Forbidden.into());
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.namespace_id;
            move |subject_id| {
                let msg = Authz {
                    namespace_ids: vec![namespace_id],
                    subject: subject_id,
                    object: format!("namespace.{}", namespace_id),
                    action: "execute".to_owned(),
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        if res? {
                            Ok(())
                        } else {
                            Err(rpc::error::Error::Forbidden)?
                        }
                    })
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_object_attr::delete::Delete::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("abac object delete res: {:?}", res);

                        Ok(Response::from(res?))
                    })
            }
        })
}
