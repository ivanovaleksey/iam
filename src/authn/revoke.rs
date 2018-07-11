use actix_web::{HttpMessage, HttpRequest, HttpResponse, Path};
use futures::future::{self, Future};
use jsonwebtoken;
use uuid::Uuid;

use actors::db;
use authn::{self, jwt};
use AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<'a> {
    pub refresh_token: &'a str,
}

impl<'a> Response<'a> {
    pub fn new(refresh_token: &'a str) -> Self {
        Response { refresh_token }
    }
}

pub fn call(
    (req, path): (HttpRequest<AppState>, Path<String>),
) -> impl Future<Item = HttpResponse, Error = authn::Error> {
    use extract_authorization_header;

    let meta = req.state().rpc_meta.clone();

    let auth_header = if let Ok(header) = extract_authorization_header(req.headers()) {
        if let Some(v) = header {
            Ok(v.to_owned())
        } else {
            Err(authn::Error::Forbidden)
        }
    } else {
        Err(authn::Error::Unauthorized)
    };

    future::result(auth_header)
        .and_then(move |jwt| {
            let key = path.into_inner();
            let account_id: Uuid = if key == "me" {
                let data = jsonwebtoken::dangerous_unsafe_decode::<jwt::RefreshToken>(&jwt)
                    .map_err(|_| authn::Error::Unauthorized)?;
                data.claims.sub
            } else {
                Uuid::parse_str(&key).map_err(|_| authn::Error::NotFound)?
            };

            Ok((jwt, account_id))
        })
        .and_then({
            let db = meta.db.clone().unwrap();
            move |(jwt, account_id)| {
                let msg = db::refresh_token::find::FindWithAccount(account_id);
                db.send(msg)
                    .from_err()
                    .and_then(|res| res.map_err(|_| authn::Error::NotFound))
                    .and_then(move |(token, account)| {
                        if account.disabled_at.is_some() {
                            Err(authn::Error::Forbidden)
                        } else {
                            Ok((jwt, token))
                        }
                    })
            }
        })
        .and_then(|(jwt, refresh_token)| {
            let key = refresh_token
                .keys
                .get(0)
                .ok_or_else(|| authn::Error::InternalError)?;

            jwt::RefreshToken::decode(&jwt, key).map_err(|_| authn::Error::Unauthorized)
        })
        .and_then({
            let db = meta.db.unwrap();
            |old_token| {
                use models::NewRefreshToken;

                let changeset = NewRefreshToken::try_new(old_token.sub)
                    .map_err(|_| authn::Error::InternalError);

                future::result(changeset).and_then(move |changeset| {
                    let msg = db::refresh_token::update::Update(changeset);
                    db.send(msg).from_err().and_then(|res| {
                        let new_token = res.map_err(|_| authn::Error::InternalError)?;
                        Ok((old_token, new_token))
                    })
                })
            }
        })
        .and_then(|(old_token, new_token)| {
            let payload = jwt::RefreshToken::new(old_token.aud, old_token.sub);

            let key = new_token
                .keys
                .get(0)
                .ok_or_else(|| authn::Error::InternalError)?;

            let refresh_token = jwt::RefreshToken::encode(&payload, key)?;

            Ok(HttpResponse::Ok().json(Response::new(&refresh_token)))
        })
}
