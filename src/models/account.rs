use serde_json::Value;
use uuid::Uuid;

use schema::account;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "account"]
pub struct Account {
    pub id: Uuid,
    pub enabled: bool,
    pub constraints: Value,
}

#[derive(AsChangeset, Insertable, Debug)]
#[table_name = "account"]
pub struct NewAccount {
    pub enabled: bool,
}
