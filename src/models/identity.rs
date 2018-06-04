use chrono::NaiveDateTime;
use uuid::Uuid;

use actors::db;
use models::Namespace;
use schema::identity;

#[derive(Associations, Identifiable, Queryable, Debug, Deserialize)]
#[belongs_to(Namespace, foreign_key = "provider")]
#[primary_key(provider, label, uid)]
#[table_name = "identity"]
pub struct Identity {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
    pub account_id: Uuid,
    pub created_at: NaiveDateTime,
}

#[derive(AsChangeset, Insertable, Debug)]
#[table_name = "identity"]
pub struct NewIdentity {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
    pub account_id: Uuid,
}

impl From<db::identity::insert::Insert> for NewIdentity {
    fn from(msg: db::identity::insert::Insert) -> Self {
        NewIdentity {
            provider: msg.provider,
            label: msg.label,
            uid: msg.uid,
            account_id: msg.account_id,
        }
    }
}
