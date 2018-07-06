use abac::types::AbacAttribute;
use chrono::NaiveDateTime;
use diesel;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{authz::Authz, identity};
use models::{identity::PrimaryKey, Identity};
use rpc;
use settings;

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
}

#[derive(Debug, Serialize)]
pub struct Response {
    provider: Uuid,
    label: String,
    uid: String,
    account_id: Uuid,
    created_at: NaiveDateTime,
}

impl From<Identity> for Response {
    fn from(identity: Identity) -> Self {
        Response {
            provider: identity.provider,
            label: identity.label,
            uid: identity.uid,
            account_id: identity.account_id,
            created_at: identity.created_at,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.provider;
            move |subject_id| {
                let iam_namespace_id = settings::iam_namespace_id();

                let msg = Authz {
                    namespace_ids: vec![iam_namespace_id],
                    subject: vec![AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", subject_id),
                    }],
                    object: vec![
                        AbacAttribute {
                            namespace_id: iam_namespace_id,
                            key: "uri".to_owned(),
                            value: format!("namespace/{}", namespace_id),
                        },
                        AbacAttribute {
                            namespace_id: iam_namespace_id,
                            key: "type".to_owned(),
                            value: "identity".to_owned(),
                        },
                    ],
                    action: vec![AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "operation".to_owned(),
                        value: "create".to_owned(),
                    }],
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();
            let req = req.clone();

            // Find existing identity by (provider, label, uid) triple.
            move |_| {
                let pk = PrimaryKey {
                    provider: req.provider,
                    label: req.label,
                    uid: req.uid,
                };
                let msg = identity::find::Find(pk);

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(move |res| {
                        debug!("identity find res: {:?}", res);

                        match res {
                            Ok(record) => {
                                error!("identity already exists: {:?}", record);
                                let e = diesel::result::Error::DatabaseError(
                                    diesel::result::DatabaseErrorKind::UniqueViolation,
                                    Box::new("Identity already exists".to_owned()),
                                );
                                Err(e).map_err(rpc::error::Error::Db)?
                            }
                            Err(e) => match e {
                                diesel::result::Error::NotFound => Ok(()),
                                _ => Err(e).map_err(rpc::error::Error::Db)?,
                            },
                        }
                    })
            }
        })
        .and_then({
            let db = meta.db.unwrap();

            // Identity is not found. Create new account & linked identity.
            move |_| {
                let pk = PrimaryKey {
                    provider: req.provider,
                    label: req.label,
                    uid: req.uid,
                };
                let msg = identity::insert::Insert::IdentityWithAccount(pk);

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("identity insert res: {:?}", res);

                        let identity = res.map_err(rpc::error::Error::Db)?;
                        Ok(Response::from(identity))
                    })
            }
        })
}
