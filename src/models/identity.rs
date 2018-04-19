use chrono::NaiveDateTime;
use uuid::Uuid;

use models::Namespace;
use schema::identity;

#[derive(Associations, Identifiable, Queryable, Debug)]
#[belongs_to(Namespace, foreign_key = "provider")]
#[primary_key(provider, label, uid)]
#[table_name = "identity"]
pub struct Identity {
    provider: Uuid,
    label: String,
    uid: String,
    issuer_id: Uuid,
    account_id: Uuid,
    issued_at: NaiveDateTime,
}
