use diesel;
use jsonrpc;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Fail, PartialEq)]
pub enum Error {
    #[fail(display = "Forbidden")]
    Forbidden,

    #[fail(display = "{}", _0)]
    Db(#[cause] diesel::result::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Error::Db(e)
    }
}

impl From<Error> for jsonrpc::Error {
    fn from(e: Error) -> Self {
        let code = match e {
            Error::Db(ref e) => match *e {
                diesel::result::Error::NotFound => 404,
                _ => 422,
            },
            Error::Forbidden => 403,
        };

        jsonrpc::Error {
            code: jsonrpc::ErrorCode::ServerError(code),
            message: e.to_string(),
            data: None,
        }
    }
}
