use chrono::NaiveDateTime;
use diesel;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db;
use models::Identity;
use rpc;

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
            let req = req.clone();

            // Find existing identity by (provider, label, uid) triple.
            move |_subject_id| {
                let msg = db::identity::find::Find {
                    provider: req.provider,
                    label: req.label,
                    uid: req.uid,
                };

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
            let db = meta.db.clone().unwrap();

            // Identity is not found. Create new account.
            move |_| {
                let msg = db::account::insert::Insert { enabled: true };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(move |res| {
                        debug!("account insert res: {:?}", res);

                        let account = res.map_err(rpc::error::Error::Db)?;
                        Ok(account.id)
                    })
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();

            // Create identity linked to the created account.
            move |account_id| {
                let msg = db::identity::insert::Insert {
                    provider: req.provider,
                    label: req.label,
                    uid: req.uid,
                    account_id,
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("identity insert res: {:?}", res);

                        let iden = res.map_err(rpc::error::Error::Db)?;
                        Ok(Response::from(iden))
                    })
            }
        })
}
