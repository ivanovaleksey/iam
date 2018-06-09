use serde_json::Value;
use uuid::Uuid;

use actors::db;
use schema::account;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "account"]
pub struct Account {
    pub id: Uuid,
    pub enabled: bool,
    pub constraints: Value,
}

#[derive(Clone, Copy, AsChangeset, Insertable, Debug)]
#[table_name = "account"]
pub struct NewAccount {
    pub enabled: bool,
}

impl From<db::account::insert::Insert> for NewAccount {
    fn from(msg: db::account::insert::Insert) -> Self {
        NewAccount {
            enabled: msg.enabled,
        }
    }
}
