use abac::types::AbacAttribute;
use diesel;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{authz::Authz, identity};
use models::identity::PrimaryKey;
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
}

pub type Response = rpc::identity::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    let namespace_id = req.provider;

    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let msg = identity::find::Find::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(move |res| {
                        debug!("identity find res: {:?}", res);

                        let res = match res {
                            Ok(identity) => Ok(Some(identity)),
                            Err(diesel::result::Error::NotFound) => Ok(None),
                            Err(e) => Err(e),
                        };

                        let identity = res.map_err(rpc::error::Error::Db)?;
                        Ok((identity, subject_id))
                    })
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |(identity, subject_id)| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};
                use future::Either;

                let iam_namespace_id = settings::iam_namespace_id();

                if let Some(identity) = identity {
                    let pk = PrimaryKey::from(identity.clone());

                    let msg = Authz {
                        namespace_ids: vec![iam_namespace_id],
                        subject: vec![AbacAttribute::new(
                            iam_namespace_id,
                            UriKind::Account(subject_id),
                        )],
                        object: vec![
                            AbacAttribute::new(iam_namespace_id, UriKind::Namespace(namespace_id)),
                            AbacAttribute::new(iam_namespace_id, UriKind::Identity(pk)),
                        ],
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Read)],
                    };

                    let f = db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| Ok(identity));

                    Either::A(f)
                } else {
                    let msg = Authz {
                        namespace_ids: vec![iam_namespace_id],
                        subject: vec![AbacAttribute::new(
                            iam_namespace_id,
                            UriKind::Account(subject_id),
                        )],
                        object: vec![
                            AbacAttribute::new(iam_namespace_id, UriKind::Namespace(namespace_id)),
                            AbacAttribute::new(iam_namespace_id, CollectionKind::Identity),
                        ],
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Read)],
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
        .and_then(|identity| Ok(Response::from(identity)))
}
