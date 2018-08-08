use futures::Future;
use jsonrpc::BoxFuture;

use rpc;

pub mod create;
pub mod delete;
pub mod list;
pub mod read;
pub mod tree;

build_rpc_trait! {
    pub trait Rpc {
        type Metadata;

        #[rpc(meta, name = "abac_subject_attr.create")]
        fn create(&self, Self::Metadata, create::Request) -> BoxFuture<create::Response>;

        #[rpc(meta, name = "abac_subject_attr.read")]
        fn read(&self, Self::Metadata, read::Request) -> BoxFuture<read::Response>;

        #[rpc(meta, name = "abac_subject_attr.delete")]
        fn delete(&self, Self::Metadata, delete::Request) -> BoxFuture<delete::Response>;

        #[rpc(meta, name = "abac_subject_attr.list")]
        fn list(&self, Self::Metadata, list::Request) -> BoxFuture<list::Response>;

        #[rpc(meta, name = "abac_subject_attr.tree")]
        fn tree(&self, Self::Metadata, tree::Request) -> BoxFuture<tree::Response>;
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

    fn tree(&self, meta: rpc::Meta, req: tree::Request) -> BoxFuture<tree::Response> {
        Box::new(tree::call(meta, req).from_err())
    }
}
