use actix;
use actix_web;
use diesel;

use authn;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "{}", _0)]
    ActorMailbox(#[cause] actix::MailboxError),

    #[fail(display = "{}", _0)]
    Db(#[cause] diesel::result::Error),

    #[fail(display = "Invalid client")]
    InvalidClient,

    #[fail(display = "Invalid request")]
    InvalidRequest,

    #[fail(display = "Internal error")]
    InternalError,

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

impl From<authn::jwt::DecodeError> for Error {
    fn from(_e: authn::jwt::DecodeError) -> Self {
        Error::InvalidClient
    }
}

impl From<authn::jwt::EncodeError> for Error {
    fn from(_e: authn::jwt::EncodeError) -> Self {
        Error::InternalError
    }
}

impl From<Error> for actix_web::Error {
    fn from(e: Error) -> Self {
        use self::Error::*;

        match e {
            ActorMailbox(_) | Db(_) | InternalError => {
                actix_web::error::ErrorInternalServerError("")
            }
            InvalidClient => bad_request("invalid_client"),
            InvalidRequest => bad_request("invalid_request"),
            Forbidden => actix_web::error::ErrorForbidden(""),
        }
    }
}

fn bad_request(cause: &str) -> actix_web::Error {
    use actix_web::HttpResponse;

    let resp = HttpResponse::BadRequest().json(json!({ "error": cause }));
    actix_web::error::InternalError::from_response("", resp).into()
}
