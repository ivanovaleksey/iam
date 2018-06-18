use futures::Future;
use jsonrpc::BoxFuture;

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

#[allow(missing_debug_implementations)]
pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn create(&self, meta: rpc::Meta, req: create::Request) -> BoxFuture<create::Response> {
        Box::new(create::call(meta, req).from_err())
    }

    fn read(&self, meta: rpc::Meta, req: read::Request) -> BoxFuture<read::Response> {
        Box::new(read::call(meta, req).from_err())
    }

    fn delete(&self, meta: rpc::Meta, req: delete::Request) -> BoxFuture<delete::Response> {
        Box::new(delete::call(meta, req).from_err())
    }

    fn list(&self, meta: rpc::Meta, req: list::Request) -> BoxFuture<list::Response> {
        Box::new(list::call(meta, req).from_err())
    }
}
