use futures::future::{self, Future};
use jsonrpc;

use actors::db::identity;
use models::identity::PrimaryKey;
use rpc;

pub type Request = rpc::identity::create::Request;
pub type Response = rpc::identity::create::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = meta.subject.ok_or(rpc::error::Error::Forbidden.into());
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |_subject_id| {
                let msg = identity::find::Find {
                    provider: req.provider,
                    label: req.label,
                    uid: req.uid,
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("identity find res: {:?}", res);

                        let iden = res.map_err(rpc::error::Error::Db)?;
                        Ok(iden)
                    })
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();
            move |identity| {
                let msg = identity::select::Select {
                    provider: None,
                    account_id: Some(identity.account_id),
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        let items = res?;
                        Ok(items.len())
                    })
                    .and_then(move |count| {
                        use future::Either;

                        let pk = PrimaryKey {
                            provider: identity.provider,
                            label: identity.label,
                            uid: identity.uid,
                        };

                        if count == 1 {
                            // It is the last user's identity.
                            // Remove both identity and account.

                            let msg = identity::delete::Delete::IdentityWithAccount(pk);
                            let f = db
                                .send(msg)
                                .map_err(|_| jsonrpc::Error::internal_error())
                                .and_then(|res| {
                                    debug!("identity delete with account res: {:?}", res);

                                    let iden = res.map_err(rpc::error::Error::Db)?;
                                    Ok(iden)
                                });

                            Either::A(f)
                        } else {
                            let msg = identity::delete::Delete::Identity(pk);
                            let f = db
                                .send(msg)
                                .map_err(|_| jsonrpc::Error::internal_error())
                                .and_then(|res| {
                                    debug!("identity delete res: {:?}", res);

                                    let iden = res.map_err(rpc::error::Error::Db)?;
                                    Ok(iden)
                                });

                            Either::B(f)
                        }
                    })
            }
        })
        .and_then({ |identity| Ok(Response::from(identity)) })
}
