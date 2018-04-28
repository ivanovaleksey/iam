use diesel::prelude::*;
use diesel::PgConnection;
use uuid::{self, Uuid};

use std::str;

use actors::db::abac_subject;
use models::AbacSubjectAttr;
use rpc;
use rpc::error::Result;

pub type Request = rpc::ListRequest<Filter>;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Filter {
    pub namespace_id: Option<Uuid>,
    pub subject_id: Option<Uuid>,
    pub key: Option<String>,
}

impl str::FromStr for Filter {
    type Err = uuid::ParseError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let mut filter = Filter {
            namespace_id: None,
            subject_id: None,
            key: None,
        };

        for part in s.split(" AND ") {
            let mut kv = part.splitn(2, ":");
            match (kv.next(), kv.next()) {
                (Some("namespace_id"), Some(v)) => {
                    let uuid = Uuid::parse_str(v)?;
                    filter.namespace_id = Some(uuid);
                }
                (Some("subject_id"), Some(v)) => {
                    let uuid = Uuid::parse_str(v)?;
                    filter.subject_id = Some(uuid);
                }
                (Some("key"), Some(v)) => {
                    filter.key = Some(v.to_owned());
                }
                _ => {}
            }
        }

        Ok(filter)
    }
}

#[derive(Debug, Serialize)]
pub struct Response(Vec<rpc::abac_subject::read::Response>);

impl From<Vec<AbacSubjectAttr>> for Response {
    fn from(items: Vec<AbacSubjectAttr>) -> Self {
        let items = items.into_iter().map(From::from).collect();
        Response(items)
    }
}

pub fn call(conn: &PgConnection, msg: abac_subject::List) -> Result<Vec<AbacSubjectAttr>> {
    use schema::abac_subject_attr::dsl::*;

    let mut query = abac_subject_attr.into_boxed();

    if let Some(namespace) = msg.namespace_id {
        query = query.filter(namespace_id.eq(namespace));
    }

    if let Some(subject) = msg.subject_id {
        query = query.filter(subject_id.eq(subject));
    }

    if let Some(k) = msg.key {
        query = query.filter(key.eq(k));
    }

    let items = query.load(conn)?;

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json;

    #[test]
    fn deserialize_filter_with_all_fields() {
        let filter = Filter {
            namespace_id: Some(Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap()),
            subject_id: Some(Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap()),
            key: Some("role".to_owned()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND subject_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420 AND key:role"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_namespace_id() {
        let filter = Filter {
            namespace_id: None,
            subject_id: Some(Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap()),
            key: Some("role".to_owned()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"subject_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420 AND key:role"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_subject_id() {
        let filter = Filter {
            namespace_id: Some(Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap()),
            subject_id: None,
            key: Some("role".to_owned()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND key:role"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_empty_filter() {
        let filter = Filter {
            namespace_id: None,
            subject_id: None,
            key: None,
        };
        let req = Request::new(filter);
        assert_eq!(req, serde_json::from_str(r#"{"fq":""}"#).unwrap());
    }
}
