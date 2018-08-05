use abac::AbacAttribute;
use diesel;
use futures::future::{self, Either, Future};
use uuid::Uuid;

use actors::db::{authz::Authz, namespace};
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Uuid,
    pub data: RequestData,
    pub label: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RequestData {
    pub label: String,
}

pub type Response = rpc::namespace::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.id;
            move |subject_id| {
                let msg = namespace::find::Find::Active(namespace_id);
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
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Update)],
                    };

                    let f = db.send(msg).from_err().and_then(rpc::ensure_authorized);

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
            move |_| {
                let msg = namespace::update::Update {
                    id: req.id,
                    label: req.data.label,
                };
                db.send(msg).from_err().and_then(|res| {
                    debug!("namespace update res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
