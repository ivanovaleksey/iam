use chrono::NaiveDateTime;
use uuid::Uuid;

use models::Account;
use schema::refresh_token;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Account)]
#[primary_key(account_id)]
#[table_name = "refresh_token"]
pub struct RefreshToken {
    account_id: Uuid,
    algorithm: String,
    keys: Vec<u8>,
    issued_at: NaiveDateTime,
}
