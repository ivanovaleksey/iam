use chrono::{DateTime, Utc};
use uuid::Uuid;

use models::Account;
use rpc::namespace::create;
use schema::namespace;

#[derive(AsChangeset, Associations, Identifiable, Queryable, Debug, Deserialize)]
#[belongs_to(Account)]
#[table_name = "namespace"]
pub struct Namespace {
    pub id: Uuid,
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(AsChangeset, Insertable, Debug, Serialize)]
#[table_name = "namespace"]
pub struct NewNamespace {
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
}

impl From<create::Request> for NewNamespace {
    fn from(msg: create::Request) -> Self {
        NewNamespace {
            label: msg.label,
            account_id: msg.account_id,
            enabled: msg.enabled,
        }
    }
}
