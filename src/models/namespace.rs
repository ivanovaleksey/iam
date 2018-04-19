use chrono::NaiveDateTime;
use uuid::Uuid;

use models::Account;
use schema::namespace;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Account)]
#[table_name = "namespace"]
pub struct Namespace {
    pub id: Uuid,
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
    pub issued_at: NaiveDateTime,
}
