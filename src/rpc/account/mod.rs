use futures::Future;
use jsonrpc::BoxFuture;

use rpc;

mod disable;
mod read;

build_rpc_trait! {
    pub trait Rpc {
        type Metadata;

        #[rpc(meta, name = "account.read")]
        fn read(&self, Self::Metadata, read::Request) -> BoxFuture<read::Response>;

        #[rpc(meta, name = "account.disable")]
        fn disable(&self, Self::Metadata, disable::Request) -> BoxFuture<disable::Response>;
    }
}

#[allow(missing_debug_implementations)]
pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn read(&self, meta: rpc::Meta, req: read::Request) -> BoxFuture<read::Response> {
        Box::new(read::call(meta, req).from_err())
    }

    fn disable(&self, meta: rpc::Meta, req: disable::Request) -> BoxFuture<disable::Response> {
        Box::new(disable::call(meta, req).from_err())
    }
}
