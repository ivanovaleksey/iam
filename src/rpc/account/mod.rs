use futures::Future;
use jsonrpc::BoxFuture;

use rpc;

pub mod read;

build_rpc_trait! {
    pub trait Rpc {
        type Metadata;

        #[rpc(meta, name = "account.read")]
        fn read(&self, Self::Metadata, read::Request) -> BoxFuture<read::Response>;
    }
}

#[allow(missing_debug_implementations)]
pub struct RpcImpl;

impl Rpc for RpcImpl {
    type Metadata = rpc::Meta;

    fn read(&self, meta: rpc::Meta, req: read::Request) -> BoxFuture<read::Response> {
        Box::new(read::call(meta, req).from_err())
    }
}
