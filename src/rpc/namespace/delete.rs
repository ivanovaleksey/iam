use abac::types::AbacAttribute;
use diesel;
use futures::future::{self, Future};
use jsonrpc;

use actors::db::{authz::Authz, namespace};
use rpc;
use settings;

pub type Request = rpc::namespace::read::Request;
pub type Response = rpc::namespace::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let msg = namespace::find::Find::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(move |res| {
                        debug!("namespace find res: {:?}", res);

                        let res = match res {
                            Ok(namespace) => Ok(Some(namespace)),
                            Err(diesel::result::Error::NotFound) => Ok(None),
                            Err(e) => Err(e),
                        };

                        let namespace = res.map_err(rpc::error::Error::Db)?;
                        Ok((namespace, subject_id))
                    })
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();
            move |(namespace, subject_id)| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};
                use future::Either;

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

                    let f = db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
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

                    let f = db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| {
                            let e = rpc::error::Error::Db(diesel::result::Error::NotFound);
                            Err(e.into())
                        });

                    Either::B(f)
                }
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |namespace| {
                let msg = namespace::delete::Delete { id: namespace.id };
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("namespace delete res: {:?}", res);
                        Ok(Response::from(res?))
                    })
            }
        })
}
