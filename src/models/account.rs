use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use schema::account;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "account"]
pub struct Account {
    pub id: Uuid,
    pub constraints: Value,
    pub disabled_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
