use abac::AbacAttribute;
use diesel;
use futures::future::{self, Either, Future};

use actors::db::{authz::Authz, namespace};
use rpc;
use settings;

pub type Request = rpc::namespace::read::Request;
pub type Response = rpc::namespace::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let msg = namespace::find::Find::Active(req.id);
                db.send(msg).from_err().and_then(move |res| {
                    debug!("namespace find res: {:?}", res);

                    let namespace = match res {
                        Ok(namespace) => Ok(Some(namespace)),
                        Err(diesel::result::Error::NotFound) => Ok(None),
                        Err(e) => Err(e),
                    }?;

                    Ok((namespace, subject_id))
                })
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();
            move |(namespace, subject_id)| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

                if let Some(namespace) = namespace {
                    let msg = Authz {
                        namespace_ids: vec![iam_namespace_id],
                        subject: vec![AbacAttribute::new(
                            iam_namespace_id,
                            UriKind::Account(subject_id),
                        )],
                        object: vec![AbacAttribute::new(
                            iam_namespace_id,
                            UriKind::Namespace(namespace.id),
                        )],
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Delete)],
                    };

                    let f = db
                        .send(msg)
                        .from_err()
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| Ok(namespace));

                    Either::A(f)
                } else {
                    let msg = Authz {
                        namespace_ids: vec![iam_namespace_id],
                        subject: vec![AbacAttribute::new(
                            iam_namespace_id,
                            UriKind::Account(subject_id),
                        )],
                        object: vec![AbacAttribute::new(
                            iam_namespace_id,
                            CollectionKind::Namespace,
                        )],
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Delete)],
                    };

                    let f = db
                        .send(msg)
                        .from_err()
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| Err(diesel::result::Error::NotFound.into()));

                    Either::B(f)
                }
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |namespace| {
                let msg = namespace::delete::Delete { id: namespace.id };
                db.send(msg).from_err().and_then(|res| {
                    debug!("namespace delete res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
