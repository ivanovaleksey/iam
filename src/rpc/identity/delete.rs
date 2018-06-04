use futures::future::{self, Future};
use jsonrpc;

use actors::db::identity;
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
                        if items.len() > 1 {
                            Ok(identity)
                        } else {
                            Err(jsonrpc::Error {
                                message: "Cannot delete last identity".to_owned(),
                                code: jsonrpc::ErrorCode::ServerError(999),
                                data: None,
                            })
                        }
                    })
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |identity| {
                let msg = identity::delete::Delete {
                    provider: identity.provider,
                    label: identity.label,
                    uid: identity.uid,
                };
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("identity delete res: {:?}", res);

                        Ok(Response::from(res?))
                    })
            }
        })
}
