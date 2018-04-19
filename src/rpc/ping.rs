use rpc::error::Result;

build_rpc_trait! {
    pub trait Rpc {
        #[rpc(name = "ping")]
        fn ping(&self) -> Result<String>;
    }
}

pub struct RpcImpl;

impl Rpc for RpcImpl {
    fn ping(&self) -> Result<String> {
        Ok("pong".to_owned())
    }
}
