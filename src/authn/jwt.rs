use chrono::naive::serde::ts_seconds;
use chrono::{NaiveDateTime, Utc};
use frank_jwt;
use jsonwebtoken;
use serde_json;
use uuid::Uuid;

use std::fmt;

use authn::AuthKey;

const ISSUER: &str = "iam.netology-group.services";

#[derive(Debug)]
pub struct RawToken<'a> {
    pub kind: RawTokenKind<'a>,
    pub value: &'a str,
}

#[derive(Debug)]
pub enum RawTokenKind<'a> {
    Iam,
    Client(&'a AuthKey),
}

impl<'a> RawToken<'a> {
    fn public_key(&self) -> Option<String> {
        use self::RawTokenKind::*;

        let settings = get_settings!();
        match self.kind {
            Iam => Some(settings.authentication.key.to_owned()),
            Client(auth_key) => settings.providers.get(auth_key).map(|v| v.key.to_owned()),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct AccessToken {
    pub aud: String,
    pub iss: String,
    #[serde(with = "ts_seconds")]
    pub exp: NaiveDateTime,
    #[serde(with = "ts_seconds")]
    pub iat: NaiveDateTime,
    pub sub: Uuid,
}

impl AccessToken {
    pub fn new(aud: String, exp: u32, sub: Uuid) -> Self {
        let now = Utc::now().timestamp();

        AccessToken {
            aud,
            iss: ISSUER.to_owned(),
            exp: NaiveDateTime::from_timestamp(now + i64::from(exp), 0),
            iat: NaiveDateTime::from_timestamp(now, 0),
            sub,
        }
    }

    pub fn decode(token: &RawToken) -> Result<AccessToken, DecodeError> {
        let key = token
            .public_key()
            .ok_or_else(|| DecodeError::UnknownIssuer)?;

        if let Ok((_header, payload)) =
            frank_jwt::decode(&token.value.to_owned(), &key, frank_jwt::Algorithm::ES256)
        {
            serde_json::from_value(payload).map_err(|_| DecodeError::InvalidPayload)
        } else {
            Err(DecodeError::InvalidSignature)
        }
    }

    pub fn encode(payload: AccessToken) -> Result<String, EncodeError> {
        let settings = get_settings!();

        frank_jwt::encode(
            json!({}),
            &settings.tokens.key,
            &serde_json::to_value(payload).unwrap(),
            frank_jwt::Algorithm::ES256,
        ).map_err(|_| EncodeError)
    }

    pub fn default_expires_in() -> u16 {
        let settings = get_settings!();
        settings.tokens.expires_in
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "AccessToken {{ aud: {}, iss: {}, exp: {}, iat: {}, sub: {} }}",
            self.aud, self.iss, self.exp, self.iat, self.sub
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub aud: String,
    pub iss: String,
    #[serde(with = "ts_seconds")]
    pub iat: NaiveDateTime,
    pub sub: Uuid,
}

impl RefreshToken {
    pub fn new(aud: String, sub: Uuid) -> Self {
        let now = Utc::now().timestamp();

        RefreshToken {
            aud,
            iss: ISSUER.to_owned(),
            iat: NaiveDateTime::from_timestamp(now, 0),
            sub,
        }
    }

    pub fn decode(token: &str, key: &[u8]) -> Result<RefreshToken, ()> {
        use jsonwebtoken::{Algorithm, Validation};
        jsonwebtoken::decode(token, key, &Validation::new(Algorithm::HS256))
            .map(|data| data.claims)
            .map_err(|_| ())
    }

    pub fn encode(payload: &RefreshToken, key: &[u8]) -> Result<String, EncodeError> {
        let header = jsonwebtoken::Header::default();
        jsonwebtoken::encode(&header, payload, key).map_err(|_| EncodeError)
    }
}

#[derive(Debug, Fail, PartialEq)]
pub enum DecodeError {
    #[fail(display = "Invalid signature")]
    InvalidSignature,

    #[fail(display = "Invalid payload")]
    InvalidPayload,

    #[fail(display = "Unknown issuer")]
    UnknownIssuer,
}

#[derive(Debug)]
pub struct EncodeError;

#[derive(Debug)]
pub struct Validator {
    pub exp: NaiveDateTime,
}

impl Validator {
    pub fn call(&self, token: &AccessToken) -> bool {
        self.exp < token.exp
    }
}

impl Default for Validator {
    fn default() -> Self {
        use chrono::Utc;
        let now = Utc::now();

        Validator {
            exp: NaiveDateTime::from_timestamp(now.timestamp(), 0),
        }
    }
}
