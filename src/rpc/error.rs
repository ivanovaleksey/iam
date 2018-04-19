use jsonrpc;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Fail)]
#[fail(display = "IAM error")]
pub struct Error;

impl From<Error> for jsonrpc::Error {
    fn from(_e: Error) -> Self {
        jsonrpc::Error::internal_error()
    }
}
