use abac::types::AbacAttribute;
use diesel;
use futures::future::{self, Either, Future};

use actors::db::{authz::Authz, identity};
use models::identity::PrimaryKey;
use rpc;
use settings;

pub type Request = rpc::identity::create::Request;
pub type Response = rpc::identity::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    let namespace_id = req.id.provider;

    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let msg = identity::find::Find(req.id);
                db.send(msg).from_err().and_then(move |res| {
                    debug!("identity find res: {:?}", res);

                    let identity = match res {
                        Ok(identity) => Ok(Some(identity)),
                        Err(diesel::result::Error::NotFound) => Ok(None),
                        Err(e) => Err(e),
                    }?;

                    Ok((identity, subject_id))
                })
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |(identity, subject_id)| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

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
                        .from_err()
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
                        .from_err()
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| Err(diesel::result::Error::NotFound.into()));

                    Either::B(f)
                }
            }
        })
        .and_then(|identity| Ok(Response::from(identity)))
}
