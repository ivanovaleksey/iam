use chrono::{DateTime, Utc};
use uuid::Uuid;

use models::Account;
use schema::namespace;

#[derive(AsChangeset, Associations, Identifiable, Queryable, Debug, Deserialize)]
#[belongs_to(Account)]
#[table_name = "namespace"]
pub struct Namespace {
    pub id: Uuid,
    pub label: String,
    pub account_id: Uuid,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(AsChangeset, Insertable, Debug, Serialize)]
#[table_name = "namespace"]
pub struct NewNamespace {
    pub label: String,
    pub account_id: Uuid,
}
