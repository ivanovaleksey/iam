use abac::types::AbacAttribute;
use diesel;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{authz::Authz, namespace};
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Uuid,
    pub label: Option<String>,
    pub enabled: Option<bool>,
}

pub type Response = rpc::namespace::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.id;
            move |subject_id| {
                let msg = namespace::find::Find::Any(namespace_id);
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
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Update)],
                    };

                    let f = db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| Ok(()));

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
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Update)],
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
            move |_| {
                let msg = namespace::update::Update::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("namespace update res: {:?}", res);
                        Ok(Response::from(res?))
                    })
            }
        })
}
