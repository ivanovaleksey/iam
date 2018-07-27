use actix;
use diesel;
use jsonrpc;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    ActorMailbox(#[cause] actix::MailboxError),

    #[fail(display = "{}", _0)]
    Db(#[cause] diesel::result::Error),

    #[fail(display = "Bad request")]
    BadRequest,

    #[fail(display = "Forbidden")]
    Forbidden,
}

impl From<actix::MailboxError> for Error {
    fn from(e: actix::MailboxError) -> Self {
        Error::ActorMailbox(e)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Error::Db(e)
    }
}

macro_rules! server_error {
    ($code:expr, $error:expr) => {
        jsonrpc::Error {
            code: jsonrpc::ErrorCode::ServerError($code),
            message: $error.to_string(),
            data: None,
        }
    };
}

impl From<Error> for jsonrpc::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::ActorMailbox(_) => jsonrpc::Error::internal_error(),
            Error::Db(ref e) => match *e {
                diesel::result::Error::NotFound => server_error!(404, e),
                _ => server_error!(422, e),
            },
            Error::BadRequest => server_error!(400, e),
            Error::Forbidden => server_error!(403, e),
        }
    }
}
