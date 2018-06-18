use futures::future::{self, FutureResult};

use rpc::error::Error;

build_rpc_trait! {
    pub trait Rpc {
        #[rpc(name = "ping")]
        fn ping(&self) -> FutureResult<String, Error>;
    }
}

#[allow(missing_debug_implementations)]
pub struct RpcImpl;

impl Rpc for RpcImpl {
    fn ping(&self) -> FutureResult<String, Error> {
        future::ok("pong".to_owned())
    }
}
