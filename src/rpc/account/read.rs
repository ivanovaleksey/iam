use abac::types::AbacAttribute;
use diesel;
use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use actors::db::{account, authz::Authz};
use models::Account;
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub id: Uuid,
}

impl From<Account> for Response {
    fn from(account: Account) -> Self {
        Response { id: account.id }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let msg = account::find::Find::from(req);
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
                use future::Either;

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

                    let f = db.send(msg)
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

                    let f = db.send(msg)
                        .from_err()
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| Err(diesel::result::Error::NotFound.into()));

                    Either::B(f)
                }
            }
        })
        .and_then(|account| Ok(Response::from(account)))
        .from_err()
}
