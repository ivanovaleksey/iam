use futures::Future;
use jsonrpc::BoxFuture;

use rpc;

pub mod create;
pub mod delete;
pub mod list;
pub mod read;
pub mod update;

build_rpc_trait! {
    pub trait Rpc {
        type Metadata;

        #[rpc(meta, name = "namespace.create")]
        fn create(&self, Self::Metadata, create::Request) -> BoxFuture<create::Response>;

        #[rpc(meta, name = "namespace.read")]
        fn read(&self, Self::Metadata, read::Request) -> BoxFuture<read::Response>;

        #[rpc(meta, name = "namespace.update")]
        fn update(&self, Self::Metadata, update::Request) -> BoxFuture<update::Response>;

        #[rpc(meta, name = "namespace.delete")]
        fn delete(&self, Self::Metadata, delete::Request) -> BoxFuture<delete::Response>;

        #[rpc(meta, name = "namespace.list")]
        fn list(&self, Self::Metadata, list::Request) -> BoxFuture<list::Response>;
    }
}

#[derive(Clone, Copy)]
pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn create(&self, meta: rpc::Meta, req: create::Request) -> BoxFuture<create::Response> {
        Box::new(create::call(meta, req).from_err())
    }

    fn read(&self, meta: rpc::Meta, req: read::Request) -> BoxFuture<read::Response> {
        Box::new(read::call(meta, req).from_err())
    }

    fn update(&self, meta: rpc::Meta, req: update::Request) -> BoxFuture<update::Response> {
        Box::new(update::call(meta, req).from_err())
    }

    fn delete(&self, meta: rpc::Meta, req: delete::Request) -> BoxFuture<delete::Response> {
        Box::new(delete::call(meta, req).from_err())
    }

    fn list(&self, meta: rpc::Meta, req: list::Request) -> BoxFuture<list::Response> {
        Box::new(list::call(meta, req).from_err())
    }
}
