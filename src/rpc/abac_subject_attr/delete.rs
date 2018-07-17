use abac::types::AbacAttribute;
use futures::future::{self, Future};
use jsonrpc;

use actors::db::{abac_subject_attr, authz::Authz};
use rpc;
use settings;

pub type Request = rpc::abac_subject_attr::create::Request;
pub type Response = rpc::abac_subject_attr::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.outbound.namespace_id;
            move |subject_id| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

                let msg = Authz {
                    namespace_ids: vec![iam_namespace_id],
                    subject: vec![AbacAttribute::new(
                        iam_namespace_id,
                        UriKind::Account(subject_id),
                    )],
                    object: vec![AbacAttribute::new(
                        namespace_id,
                        CollectionKind::AbacSubject,
                    )],
                    action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Delete)],
                };

                db.send(msg).from_err().and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_subject_attr::delete::Delete::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac subject delete res: {:?}", res);

                    Ok(Response::from(res?))
                })
            }
        })
        .from_err()
}
