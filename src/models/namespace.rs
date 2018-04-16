use chrono::NaiveDateTime;
use uuid::Uuid;

use models::account::Account;
use schema::namespace;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Account)]
#[table_name = "namespace"]
pub struct Namespace {
    id: Uuid,
    label: String,
    account_id: Uuid,
    enabled: bool,
    issued_at: NaiveDateTime,
}
