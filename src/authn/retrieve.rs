use actix_web::{self, HttpMessage, HttpRequest, HttpResponse, Path};
use futures::future::{self, Either, Future};

use actors::db;
use authn::{self, jwt, AuthKey};
use AppState;

#[derive(Debug, Deserialize, PartialEq)]
struct Payload {
    pub grant_type: String,
    pub client_token: String,
    #[serde(default = "jwt::AccessToken::default_expires_in")]
    pub expires_in: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<'a> {
    pub access_token: &'a str,
    pub refresh_token: &'a str,
    pub expires_in: u16,
    pub token_type: &'a str,
}

impl<'a> Response<'a> {
    pub fn new(access_token: &'a str, refresh_token: &'a str, expires_in: u16) -> Self {
        use TOKEN_TYPE;
        Response {
            token_type: TOKEN_TYPE,
            access_token,
            refresh_token,
            expires_in,
        }
    }
}

pub fn call(
    (req, path): (HttpRequest<AppState>, Path<AuthKey>),
) -> impl Future<Item = HttpResponse, Error = authn::Error> {
    use actix_web::FromRequest;

    let meta = req.state().rpc_meta.clone();
    let content_type = req
        .headers()
        .get("Content-Type")
        .expect("Content-Type is not specified")
        .to_str()
        .map(|s| s.to_owned())
        .map_err(|_| authn::Error::InternalError);

    future::result(content_type)
        .and_then(move |content_type| {
            let f = match content_type.as_ref() {
                "application/x-www-form-urlencoded" => {
                    let f = actix_web::Form::<Payload>::extract(&req).map(|v| v.into_inner());
                    Either::A(f)
                }
                "application/json" => {
                    let f = actix_web::Json::<Payload>::extract(&req).map(|v| v.into_inner());
                    Either::B(f)
                }
                _ => unreachable!(),
            };

            f.then(|res| {
                if let Ok(payload) = res {
                    let settings = get_settings!();

                    if payload.grant_type == "client_credentials"
                        && payload.expires_in <= settings.tokens.expires_in_max
                    {
                        return Ok(payload);
                    }
                }

                Err(authn::Error::InvalidRequest)
            })
        })
        .and_then(|payload| {
            let auth_key = path.into_inner();

            let client_token = {
                let raw_token = jwt::RawToken {
                    kind: jwt::RawTokenKind::Client(&auth_key),
                    value: &payload.client_token,
                };
                jwt::AccessToken::decode(&raw_token)?
            };

            let validator = jwt::Validator::default();
            if validator.call(&client_token) {
                Ok((payload.expires_in, client_token.sub, auth_key))
            } else {
                Err(authn::Error::InvalidClient)
            }
        })
        .and_then({
            let db = meta.db.clone().unwrap();
            move |(expires_in, sub, auth_key)| {
                let AuthKey { provider, label } = auth_key;
                let msg = db::namespace::find::Find::ByLabel(provider);
                db.send(msg).from_err().and_then(move |res| {
                    let namespace = res?;
                    Ok((expires_in, sub, label, namespace))
                })
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |(expires_in, sub, label, namespace)| {
                use models::identity::PrimaryKey;

                let pk = PrimaryKey {
                    provider: namespace.id,
                    label,
                    uid: sub.to_string(),
                };
                let msg = db::identity::upsert::Upsert(pk);
                db.send(msg).from_err().and_then(|res| Ok(res?)).and_then(
                    move |(identity, account, refresh_token)| {
                        if account.disabled_at.is_some() {
                            Err(authn::Error::Forbidden)
                        } else {
                            Ok((expires_in, identity, refresh_token, namespace))
                        }
                    },
                )
            }
        })
        .and_then(|(expires_in, identity, refresh_token, namespace)| {
            let payload =
                jwt::AccessToken::new(namespace.label.clone(), expires_in, identity.account_id);
            let access_token = jwt::AccessToken::encode(payload)?;

            let payload = jwt::RefreshToken::new(namespace.label, identity.account_id);
            let key = refresh_token
                .keys
                .get(0)
                .ok_or_else(|| authn::Error::InternalError)?;
            let refresh_token = jwt::RefreshToken::encode(&payload, key)?;

            Ok(HttpResponse::Ok().json(Response::new(&access_token, &refresh_token, expires_in)))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn deserialize_retrieve_payload() {
        before_each();

        let s = r#"{
            "grant_type": "foo",
            "client_token": "bar",
            "expires_in": 10
        }"#;
        let payload = serde_json::from_str::<Payload>(s).unwrap();
        let expected = Payload {
            grant_type: "foo".to_owned(),
            client_token: "bar".to_owned(),
            expires_in: 10,
        };
        assert_eq!(payload, expected);

        let s = r#"{
            "grant_type": "foo",
            "client_token": "bar"
        }"#;
        let payload = serde_json::from_str::<Payload>(s).unwrap();
        let expected = Payload {
            grant_type: "foo".to_owned(),
            client_token: "bar".to_owned(),
            expires_in: 300,
        };
        assert_eq!(payload, expected);
    }

    fn before_each() {
        use settings;
        settings::init().unwrap();
    }
}
