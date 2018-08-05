use abac::AbacAttribute;
use chrono::{DateTime, Utc};
use diesel;
use futures::future::{self, Either, Future};
use uuid::Uuid;

use actors::db::{account, authz::Authz};
use models::Account;
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Uuid,
}

pub type Response = rpc::Response<Uuid, ResponseData>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub disabled_at: Option<DateTime<Utc>>,
}

impl From<Account> for Response {
    fn from(account: Account) -> Self {
        Response {
            id: account.id,
            data: ResponseData {
                disabled_at: account.disabled_at,
            },
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let msg = account::find::Find::Active(req.id);
                db.send(msg).from_err().and_then(move |res| {
                    debug!("account find res: {:?}", res);

                    let account = match res {
                        Ok(account) => Ok(Some(account)),
                        Err(diesel::result::Error::NotFound) => Ok(None),
                        Err(e) => Err(e),
                    }?;

                    Ok((account, subject_id))
                })
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |(account, subject_id)| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

                if let Some(account) = account {
                    let msg = Authz {
                        namespace_ids: vec![iam_namespace_id],
                        subject: vec![AbacAttribute::new(
                            iam_namespace_id,
                            UriKind::Account(subject_id),
                        )],
                        object: vec![AbacAttribute::new(
                            iam_namespace_id,
                            UriKind::Account(account.id),
                        )],
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Read)],
                    };

                    let f = db
                        .send(msg)
                        .from_err()
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| Ok(account));

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
                            CollectionKind::Account,
                        )],
                        action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Read)],
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
        .and_then(|account| Ok(Response::from(account)))
}
