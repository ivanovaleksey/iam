use actix_web::{HttpMessage, HttpRequest, HttpResponse, Path};
use futures::Future;
use jsonwebtoken;
use serde_json;
use uuid::Uuid;

use actors::db;
use authn::{self, jwt};
use AppState;

#[derive(Debug, Deserialize)]
pub struct Payload {
    #[serde(default = "jwt::AccessToken::default_expires_in")]
    pub expires_in: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<'a> {
    pub access_token: &'a str,
    pub expires_in: u16,
    pub token_type: &'a str,
}

impl<'a> Response<'a> {
    pub fn new(access_token: &'a str, expires_in: u16) -> Self {
        use TOKEN_TYPE;
        Response {
            token_type: TOKEN_TYPE,
            access_token,
            expires_in,
        }
    }
}

pub fn call(
    (req, path): (HttpRequest<AppState>, Path<String>),
) -> impl Future<Item = HttpResponse, Error = authn::Error> {
    use extract_authorization_header;

    let meta = req.state().rpc_meta.clone();
    let headers = req.headers().clone();

    req.body()
        .map_err(|_| authn::Error::InternalError)
        .and_then(|body| {
            if body.is_empty() {
                Ok(authn::jwt::AccessToken::default_expires_in())
            } else {
                if let Ok(payload) = serde_json::from_slice::<Payload>(&body) {
                    let settings = get_settings!();
                    if payload.expires_in <= settings.tokens.expires_in_max {
                        return Ok(payload.expires_in);
                    }
                }

                Err(authn::Error::BadRequest)
            }
        })
        .and_then(move |expires_in| {
            if let Ok(header) = extract_authorization_header(&headers) {
                if let Some(v) = header {
                    Ok((expires_in, v.to_owned()))
                } else {
                    Err(authn::Error::Forbidden)
                }
            } else {
                Err(authn::Error::Unauthorized)
            }
        })
        .and_then(move |(expires_in, jwt)| {
            let key = path.into_inner();
            let account_id: Uuid = if key == "me" {
                let data = jsonwebtoken::dangerous_unsafe_decode::<jwt::RefreshToken>(&jwt)
                    .map_err(|_| authn::Error::Unauthorized)?;
                data.claims.sub
            } else {
                Uuid::parse_str(&key).map_err(|_| authn::Error::NotFound)?
            };

            Ok((expires_in, jwt, account_id))
        })
        .and_then(|(expires_in, jwt, account_id)| {
            let db = meta.db.unwrap();

            let msg = db::refresh_token::find::FindWithAccount(account_id);
            db.send(msg)
                .from_err()
                .and_then(|res| res.map_err(|_| authn::Error::NotFound))
                .and_then(move |(token, account)| {
                    if account.disabled_at.is_some() {
                        Err(authn::Error::Forbidden)
                    } else {
                        Ok((expires_in, jwt, token))
                    }
                })
        })
        .and_then(|(expires_in, jwt, refresh_token)| {
            let key = refresh_token
                .keys
                .get(0)
                .ok_or_else(|| authn::Error::InternalError)?;

            let token =
                jwt::RefreshToken::decode(&jwt, key).map_err(|_| authn::Error::Unauthorized)?;
            Ok((expires_in, token))
        })
        .and_then(|(expires_in, refresh_token)| {
            let payload =
                jwt::AccessToken::new(refresh_token.aud, u32::from(expires_in), refresh_token.sub);
            let access_token = jwt::AccessToken::encode(payload)?;

            Ok(HttpResponse::Ok().json(Response::new(&access_token, expires_in)))
        })
}
