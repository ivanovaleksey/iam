use abac::types::AbacAttribute;
use chrono::{DateTime, Utc};
use diesel;
use futures::future::{self, Future};
use uuid::Uuid;

use actors::db::{authz::Authz, identity};
use models::{identity::PrimaryKey, Identity};
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: PrimaryKey,
}

pub type Response = rpc::Response<PrimaryKey, ResponseData>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub account_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<Identity> for Response {
    fn from(identity: Identity) -> Self {
        Response {
            id: PrimaryKey {
                provider: identity.provider,
                label: identity.label,
                uid: identity.uid,
            },
            data: ResponseData {
                account_id: identity.account_id,
                created_at: identity.created_at,
            },
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.id.provider;
            move |subject_id| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

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
                    action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Create)],
                };

                db.send(msg).from_err().and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();
            let pk = req.id.clone();

            // Find existing identity by (provider, label, uid) triple.
            move |_| {
                let msg = identity::find::Find(pk);

                db.send(msg).from_err().and_then(move |res| {
                    debug!("identity find res: {:?}", res);

                    match res {
                        Ok(record) => {
                            error!("identity already exists: {:?}", record);
                            let e = diesel::result::Error::DatabaseError(
                                diesel::result::DatabaseErrorKind::UniqueViolation,
                                Box::new("Identity already exists".to_owned()),
                            );
                            Err(e)?
                        }
                        Err(e) => match e {
                            diesel::result::Error::NotFound => Ok(()),
                            _ => Err(e)?,
                        },
                    }
                })
            }
        })
        .and_then({
            let db = meta.db.unwrap();

            // Identity is not found. Create new account & linked identity.
            move |_| {
                let msg = identity::insert::Insert::IdentityWithAccount(req.id);

                db.send(msg).from_err().and_then(|res| {
                    debug!("identity insert res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
