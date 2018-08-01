use abac::AbacAttribute;
use diesel;
use futures::future::{self, Future};

use actors::db::{account, authz::Authz};
use rpc;
use settings;

pub type Request = rpc::account::read::Request;
pub type Response = rpc::account::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let msg = account::find::Find::Enabled(req.id);
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
            let db = meta.db.clone().unwrap();
            move |(account, subject_id)| {
                use abac_attribute::{CollectionKind, OperationKind, UriKind};

                let iam_namespace_id = settings::iam_namespace_id();

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
                    action: vec![AbacAttribute::new(iam_namespace_id, OperationKind::Update)],
                };

                db.send(msg)
                    .from_err()
                    .and_then(rpc::ensure_authorized)
                    .and_then(|_| {
                        if let Some(account) = account {
                            Ok(account)
                        } else {
                            Err(diesel::result::Error::NotFound)?
                        }
                    })
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |account| {
                let msg = account::update::Disable(account.id);
                db.send(msg)
                    .from_err()
                    .and_then(|res| Ok(Response::from(res?)))
            }
        })
}
