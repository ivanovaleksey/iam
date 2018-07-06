use abac::types::AbacAttribute;
use diesel;
use futures::future::{self, Either, Future};
use jsonrpc;

use actors::db::{authz::Authz, identity};
use models::identity::PrimaryKey;
use rpc;
use settings;

pub type Request = rpc::identity::read::Request;
pub type Response = rpc::identity::read::Response;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    let namespace_id = req.provider;

    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                let msg = identity::find::Find::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(move |res| {
                        debug!("identity find res: {:?}", res);

                        let res = match res {
                            Ok(identity) => Ok(Some(identity)),
                            Err(diesel::result::Error::NotFound) => Ok(None),
                            Err(e) => Err(e),
                        };

                        let identity = res.map_err(rpc::error::Error::Db)?;
                        Ok((identity, subject_id))
                    })
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();
            move |(identity, subject_id)| {
                let iam_namespace_id = settings::iam_namespace_id();

                if let Some(identity) = identity {
                    let pk = PrimaryKey::from(identity.clone());

                    let msg = Authz {
                        namespace_ids: vec![iam_namespace_id],
                        subject: vec![AbacAttribute {
                            namespace_id: iam_namespace_id,
                            key: "uri".to_owned(),
                            value: format!("account/{}", subject_id),
                        }],
                        object: vec![AbacAttribute {
                            namespace_id,
                            key: "uri".to_owned(),
                            value: format!("identity/{}", pk),
                        }],
                        action: vec![AbacAttribute {
                            namespace_id: iam_namespace_id,
                            key: "operation".to_owned(),
                            value: "delete".to_owned(),
                        }],
                    };

                    let f = db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| Ok(identity));

                    Either::A(f)
                } else {
                    let msg = Authz {
                        namespace_ids: vec![iam_namespace_id],
                        subject: vec![AbacAttribute {
                            namespace_id: iam_namespace_id,
                            key: "uri".to_owned(),
                            value: format!("account/{}", subject_id),
                        }],
                        object: vec![AbacAttribute {
                            namespace_id,
                            key: "type".to_owned(),
                            value: "identity".to_owned(),
                        }],
                        action: vec![AbacAttribute {
                            namespace_id: iam_namespace_id,
                            key: "operation".to_owned(),
                            value: "delete".to_owned(),
                        }],
                    };

                    let f = db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(rpc::ensure_authorized)
                        .and_then(|_| {
                            let e = rpc::error::Error::Db(diesel::result::Error::NotFound);
                            Err(e.into())
                        });

                    Either::B(f)
                }
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |identity| {
                let msg = identity::select::Select::ByAccountId(identity.account_id);

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        let items = res?;
                        Ok(items.len())
                    })
                    .and_then(move |count| {
                        let pk = PrimaryKey {
                            provider: identity.provider,
                            label: identity.label,
                            uid: identity.uid,
                        };

                        if count == 1 {
                            // It is the last user's identity.
                            // Remove both identity and account.

                            let msg = identity::delete::Delete::IdentityWithAccount(pk);
                            let f = db.send(msg)
                                .map_err(|_| jsonrpc::Error::internal_error())
                                .and_then(|res| {
                                    debug!("identity delete with account res: {:?}", res);

                                    let identity = res.map_err(rpc::error::Error::Db)?;
                                    Ok(identity)
                                });

                            Either::A(f)
                        } else {
                            let msg = identity::delete::Delete::Identity(pk);
                            let f = db.send(msg)
                                .map_err(|_| jsonrpc::Error::internal_error())
                                .and_then(|res| {
                                    debug!("identity delete res: {:?}", res);

                                    let identity = res.map_err(rpc::error::Error::Db)?;
                                    Ok(identity)
                                });

                            Either::B(f)
                        }
                    })
            }
        })
        .and_then({ |identity| Ok(Response::from(identity)) })
}
