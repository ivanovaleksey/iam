use chrono::NaiveDateTime;
use diesel;
use failure;
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

#[derive(Debug, SqlType, QueryId)]
#[postgres(type_name = "identity_composite_pkey")]
pub struct SqlPrimaryKey;

use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql, WriteTuple};
use diesel::sql_types::{Text, Uuid as SqlUuid};
use std::io::Write;

impl ToSql<SqlPrimaryKey, Pg> for PrimaryKey {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        WriteTuple::<(SqlUuid, Text, Text)>::write_tuple(
            &(self.provider, self.label.as_str(), self.uid.as_str()),
            out,
        )
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
