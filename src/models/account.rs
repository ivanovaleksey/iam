use serde_json::Value;
use uuid::Uuid;

use schema::account;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "account"]
pub struct Account {
    id: Uuid,
    enabled: bool,
    constraints: Value,
}
