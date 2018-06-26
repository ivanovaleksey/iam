use chrono::NaiveDateTime;
use uuid::Uuid;

use actors::db;
use models::Account;
use schema::namespace;

#[derive(AsChangeset, Associations, Identifiable, Queryable, Debug, Deserialize)]
#[belongs_to(Account)]
#[table_name = "namespace"]
pub struct Namespace {
    pub id: Uuid,
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
}

#[derive(AsChangeset, Insertable, Debug, Serialize)]
#[table_name = "namespace"]
pub struct NewNamespace {
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
}

impl From<db::namespace::insert::Insert> for NewNamespace {
    fn from(msg: db::namespace::insert::Insert) -> Self {
        NewNamespace {
            label: msg.label,
            account_id: msg.account_id,
            enabled: msg.enabled,
        }
    }
}
