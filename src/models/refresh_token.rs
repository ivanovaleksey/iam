use chrono::{DateTime, Utc};
use uuid::Uuid;

use std::borrow::Cow;

use models::Account;
use schema::refresh_token;

const ALGORITHM: &str = "HS256";

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Account)]
#[primary_key(account_id)]
#[table_name = "refresh_token"]
pub struct RefreshToken {
    pub account_id: Uuid,
    pub algorithm: String,
    pub keys: Vec<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

#[derive(AsChangeset, Identifiable, Insertable, Debug)]
#[primary_key(account_id)]
#[table_name = "refresh_token"]
pub struct NewRefreshToken {
    pub account_id: Uuid,
    pub algorithm: Cow<'static, str>,
    pub keys: Vec<Vec<u8>>,
}

impl NewRefreshToken {
    pub fn try_new(account_id: Uuid) -> Result<Self, ()> {
        use ring::rand::SecureRandom;
        use SYSTEM_RANDOM;

        let mut buf = vec![0; 64];
        SYSTEM_RANDOM.fill(&mut buf).map_err(|_| ())?;

        Ok(NewRefreshToken {
            account_id,
            algorithm: ALGORITHM.into(),
            keys: vec![buf],
        })
    }
}
