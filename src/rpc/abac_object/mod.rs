use futures::Future;
use jsonrpc::{self, BoxFuture};

use actors::db::abac_object;
use rpc;

pub mod create;
pub mod delete;
pub mod list;
pub mod read;

build_rpc_trait! {
    pub trait Rpc {
        type Metadata;

        #[rpc(meta, name = "abac_object.create")]
        fn create(&self, Self::Metadata, create::Request) -> BoxFuture<create::Response>;

        #[rpc(meta, name = "abac_object.read")]
        fn read(&self, Self::Metadata, read::Request) -> BoxFuture<read::Response>;

        #[rpc(meta, name = "abac_object.delete")]
        fn delete(&self, Self::Metadata, delete::Request) -> BoxFuture<delete::Response>;

        #[rpc(meta, name = "abac_object.list")]
        fn list(&self, Self::Metadata, list::Request) -> BoxFuture<list::Response>;
    }
}

pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn create(&self, meta: rpc::Meta, req: create::Request) -> BoxFuture<create::Response> {
        let msg = abac_object::Create::from(req);
        let fut = meta.db
            .unwrap()
            .send(msg)
            .map_err(|_| jsonrpc::Error::internal_error())
            .and_then(|res| {
                debug!("abac obj create res: {:?}", res);
                match res {
                    Ok(res) => Ok(create::Response::from(res)),
                    Err(e) => Err(e.into()),
                }
            });

        Box::new(fut)
    }

    fn read(&self, meta: rpc::Meta, req: read::Request) -> BoxFuture<read::Response> {
        let msg = abac_object::Read::from(req);
        let fut = meta.db
            .unwrap()
            .send(msg)
            .map_err(|_| jsonrpc::Error::internal_error())
            .and_then(|res| {
                debug!("abac obj read res: {:?}", res);
                match res {
                    Ok(res) => Ok(read::Response::from(res)),
                    Err(e) => Err(e.into()),
                }
            });

        Box::new(fut)
    }

    fn delete(&self, meta: rpc::Meta, req: delete::Request) -> BoxFuture<delete::Response> {
        let msg = abac_object::Delete::from(req);
        let fut = meta.db
            .unwrap()
            .send(msg)
            .map_err(|_| jsonrpc::Error::internal_error())
            .and_then(|res| {
                debug!("abac obj delete res: {:?}", res);
                match res {
                    Ok(res) => Ok(delete::Response::from(res)),
                    Err(e) => Err(e.into()),
                }
            });

        Box::new(fut)
    }

    fn list(&self, meta: rpc::Meta, req: list::Request) -> BoxFuture<list::Response> {
        let msg = abac_object::List::from(req);
        let fut = meta.db
            .unwrap()
            .send(msg)
            .map_err(|_| jsonrpc::Error::internal_error())
            .and_then(|res| {
                debug!("abac obj list res: {:?}", res);
                match res {
                    Ok(res) => Ok(list::Response::from(res)),
                    Err(e) => Err(e.into()),
                }
            });

        Box::new(fut)
    }
}
