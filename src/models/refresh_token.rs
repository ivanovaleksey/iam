use chrono::{DateTime, Utc};
use uuid::Uuid;

use models::Account;
use schema::refresh_token;

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

#[derive(Insertable, Debug)]
#[table_name = "refresh_token"]
pub struct NewRefreshToken {
    pub account_id: Uuid,
    pub algorithm: String,
    pub keys: Vec<Vec<u8>>,
}
