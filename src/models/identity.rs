use chrono::NaiveDateTime;
use diesel;
use uuid::Uuid;

use std::{fmt, str};

use models::Namespace;
use schema::identity;

#[derive(Clone, Debug, Serialize)]
pub struct PrimaryKey {
    pub provider: Uuid,
    pub label: String,
    pub uid: String,
}

impl PrimaryKey {
    pub fn as_tuple(&self) -> <&Identity as diesel::Identifiable>::Id {
        (&self.provider, &self.label, &self.uid)
    }
}

impl From<Identity> for PrimaryKey {
    fn from(identity: Identity) -> Self {
        PrimaryKey {
            provider: identity.provider,
            label: identity.label,
            uid: identity.uid,
        }
    }
}

impl fmt::Display for PrimaryKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.uid, self.label, self.provider)
    }
}

use failure;
impl str::FromStr for PrimaryKey {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(3, '.');

        match (parts.next(), parts.next(), parts.next()) {
            (Some(uid), Some(label), Some(provider)) => {
                let provider = Uuid::parse_str(provider)?;
                let key = PrimaryKey {
                    provider,
                    label: label.to_owned(),
                    uid: uid.to_owned(),
                };
                Ok(key)
            }
            _ => Err(failure::err_msg("Bad primary key format")),
        }
    }
}

#[derive(Associations, Identifiable, Queryable, Clone, Debug, Deserialize)]
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
