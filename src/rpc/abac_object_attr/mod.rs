use futures::future::{self, Future};
use jsonrpc::{self, BoxFuture};

use actors::db::{abac_object_attr, authz::Authz};
use rpc;

pub mod create;
pub mod delete;
pub mod list;
pub mod read;

build_rpc_trait! {
    pub trait Rpc {
        type Metadata;

        #[rpc(meta, name = "abac_object_attr.create")]
        fn create(&self, Self::Metadata, create::Request) -> BoxFuture<create::Response>;

        #[rpc(meta, name = "abac_object_attr.read")]
        fn read(&self, Self::Metadata, read::Request) -> BoxFuture<read::Response>;

        #[rpc(meta, name = "abac_object_attr.delete")]
        fn delete(&self, Self::Metadata, delete::Request) -> BoxFuture<delete::Response>;

        #[rpc(meta, name = "abac_object_attr.list")]
        fn list(&self, Self::Metadata, list::Request) -> BoxFuture<list::Response>;
    }
}

pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn create(&self, meta: rpc::Meta, req: create::Request) -> BoxFuture<create::Response> {
        let subject = meta.subject.ok_or(rpc::error::Error::Forbidden.into());
        let f = future::result(subject).and_then(|subject| {
            let msg = Authz {
                namespace_ids: vec![req.namespace_id],
                subject,
                object: format!("namespace.{}", req.namespace_id),
                action: "execute".to_owned(),
            };

            let db = meta.db.unwrap();
            db.send(msg)
                .map_err(|_| jsonrpc::Error::internal_error())
                .and_then(|res| {
                    if res? {
                        Ok(())
                    } else {
                        Err(rpc::error::Error::Forbidden)?
                    }
                })
                .and_then(move |_| {
                    let msg = abac_object_attr::Create::from(req);
                    db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(|res| {
                            debug!("abac obj create res: {:?}", res);

                            Ok(create::Response::from(res?))
                        })
                })
        });

        Box::new(f)
    }

    fn read(&self, meta: rpc::Meta, req: read::Request) -> BoxFuture<read::Response> {
        let subject = meta.subject.ok_or(rpc::error::Error::Forbidden.into());
        let f = future::result(subject).and_then(|subject| {
            let msg = Authz {
                namespace_ids: vec![req.namespace_id],
                subject,
                object: format!("namespace.{}", req.namespace_id),
                action: "execute".to_owned(),
            };

            let db = meta.db.unwrap();
            db.send(msg)
                .map_err(|_| jsonrpc::Error::internal_error())
                .and_then(|res| {
                    if res? {
                        Ok(())
                    } else {
                        Err(rpc::error::Error::Forbidden)?
                    }
                })
                .and_then(move |_| {
                    let msg = abac_object_attr::Read::from(req);
                    db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(|res| {
                            debug!("abac obj read res: {:?}", res);

                            Ok(read::Response::from(res?))
                        })
                })
        });

        Box::new(f)
    }

    fn delete(&self, meta: rpc::Meta, req: delete::Request) -> BoxFuture<delete::Response> {
        let subject = meta.subject.ok_or(rpc::error::Error::Forbidden.into());
        let f = future::result(subject).and_then(|subject| {
            let msg = Authz {
                namespace_ids: vec![req.namespace_id],
                subject,
                object: format!("namespace.{}", req.namespace_id),
                action: "execute".to_owned(),
            };

            let db = meta.db.unwrap();
            db.send(msg)
                .map_err(|_| jsonrpc::Error::internal_error())
                .and_then(|res| {
                    if res? {
                        Ok(())
                    } else {
                        Err(rpc::error::Error::Forbidden)?
                    }
                })
                .and_then(move |_| {
                    let msg = abac_object_attr::Delete::from(req);
                    db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(|res| {
                            debug!("abac obj delete res: {:?}", res);

                            Ok(delete::Response::from(res?))
                        })
                })
        });

        Box::new(f)
    }

    fn list(&self, meta: rpc::Meta, req: list::Request) -> BoxFuture<list::Response> {
        let subject = meta.subject.ok_or(rpc::error::Error::Forbidden.into());
        let f = future::result(subject).and_then(|subject| {
            let msg = Authz {
                namespace_ids: vec![req.filter.0.namespace_id],
                subject,
                object: format!("namespace.{}", req.filter.0.namespace_id),
                action: "execute".to_owned(),
            };

            let db = meta.db.unwrap();
            db.send(msg)
                .map_err(|_| jsonrpc::Error::internal_error())
                .and_then(|res| {
                    if res? {
                        Ok(())
                    } else {
                        Err(rpc::error::Error::Forbidden)?
                    }
                })
                .and_then(move |_| {
                    let msg = abac_object_attr::List::from(req);
                    db.send(msg)
                        .map_err(|_| jsonrpc::Error::internal_error())
                        .and_then(|res| {
                            debug!("abac obj list res: {:?}", res);

                            Ok(list::Response::from(res?))
                        })
                })
        });

        Box::new(f)
    }
}
