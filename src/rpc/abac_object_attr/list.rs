use diesel::prelude::*;
use diesel::PgConnection;
use uuid::{self, Uuid};

use std::str;

use actors::db::abac_object_attr;
use models::AbacObjectAttr;
use rpc;
use rpc::error::Result;

pub type Request = rpc::ListRequest<Filter>;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Filter {
    pub namespace_id: Option<Uuid>,
    pub object_id: Option<String>,
    pub key: Option<String>,
}

impl str::FromStr for Filter {
    type Err = uuid::ParseError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let mut filter = Filter {
            namespace_id: None,
            object_id: None,
            key: None,
        };

        for part in s.split(" AND ") {
            let mut kv = part.splitn(2, ":");
            match (kv.next(), kv.next()) {
                (Some("namespace_id"), Some(v)) => {
                    let uuid = Uuid::parse_str(v)?;
                    filter.namespace_id = Some(uuid);
                }
                (Some("object_id"), Some(v)) => {
                    filter.object_id = Some(v.to_owned());
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
pub struct Response(Vec<rpc::abac_object_attr::read::Response>);

impl From<Vec<AbacObjectAttr>> for Response {
    fn from(items: Vec<AbacObjectAttr>) -> Self {
        let items = items.into_iter().map(From::from).collect();
        Response(items)
    }
}

pub fn call(conn: &PgConnection, msg: abac_object_attr::List) -> Result<Vec<AbacObjectAttr>> {
    use schema::abac_object_attr::dsl::*;

    let mut query = abac_object_attr.into_boxed();

    if let Some(namespace) = msg.namespace_id {
        query = query.filter(namespace_id.eq(namespace));
    }

    if let Some(object) = msg.object_id {
        query = query.filter(object_id.eq(object));
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
            object_id: Some("foo".to_owned()),
            key: Some("type".to_owned()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:foo AND key:type"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_namespace_id() {
        let filter = Filter {
            namespace_id: None,
            object_id: Some("foo".to_owned()),
            key: Some("type".to_owned()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(r#"{"fq":"object_id:foo AND key:type"}"#).unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_object_id() {
        let filter = Filter {
            namespace_id: Some(Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap()),
            object_id: None,
            key: Some("type".to_owned()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND key:type"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_key() {
        let filter = Filter {
            namespace_id: Some(Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap()),
            object_id: Some("foo".to_owned()),
            key: None,
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:foo"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_empty_filter() {
        let filter = Filter {
            namespace_id: None,
            object_id: None,
            key: None,
        };
        let req = Request::new(filter);
        assert_eq!(req, serde_json::from_str(r#"{"fq":""}"#).unwrap());
    }
}