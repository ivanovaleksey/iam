use abac::types::AbacAttribute;
use futures::future::{self, Future};
use jsonrpc;

use actors::db::{abac_policy, authz::Authz};
use rpc;
use settings;

pub type Request = rpc::abac_policy::create::Request;
pub type Response = rpc::abac_policy::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.namespace_id;
            move |subject_id| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

                let msg = Authz {
                    namespace_ids: vec![iam_namespace_id],
                    subject: vec![AbacAttribute::new(
                        iam_namespace_id,
                        UriKind::Account(subject_id),
                    )],
                    object: vec![AbacAttribute::new(namespace_id, CollectionKind::AbacPolicy)],
                    action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Read)],
                };

                db.send(msg).from_err().and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_policy::find::Find::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac policy find res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
        .from_err()
}
