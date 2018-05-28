use diesel::prelude::*;
use diesel::PgConnection;
use serde_json;
use uuid::Uuid;

use std::str;

use actors::db::abac_object_attr;
use models::AbacObjectAttr;
use rpc;
use rpc::error::Result;

pub type Request = rpc::ListRequest<Filter>;

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct Filter {
    pub namespace_id: Uuid,
    pub object_id: Option<String>,
    pub key: Option<String>,
}

impl str::FromStr for Filter {
    type Err = rpc::ListRequestFilterError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let mut filter = Filter::default();
        let mut is_namespace_present = false;

        for part in s.split(" AND ") {
            let mut kv = part.splitn(2, ":");
            match (kv.next(), kv.next()) {
                (Some("namespace_id"), Some(v)) => {
                    let uuid = Uuid::parse_str(v)?;
                    filter.namespace_id = uuid;
                    is_namespace_present = true;
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

        if !is_namespace_present {
            use serde::de::Error;
            return Err(serde_json::Error::missing_field("namespace_id"))?;
        }

        Ok(filter)
    }
}

pub type Response = rpc::ListResponse<rpc::abac_object_attr::read::Response>;

pub fn call(conn: &PgConnection, msg: abac_object_attr::List) -> Result<Vec<AbacObjectAttr>> {
    use schema::abac_object_attr::dsl::*;

    let mut query = abac_object_attr.into_boxed();

    query = query.filter(namespace_id.eq(msg.namespace_id));

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
            namespace_id: Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap(),
            object_id: Some("namespace".to_owned()),
            key: Some("type".to_owned()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:namespace AND key:type"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_namespace_id() {
        let res = serde_json::from_str::<Request>(r#"{"fq":"object_id:namespace AND key:type"}"#);

        assert!(res.is_err());

        let err = res.unwrap_err();
        assert_eq!(
            format!("{}", err),
            "missing field `namespace_id` at line 1 column 41"
        );
    }

    #[test]
    fn deserialize_filter_without_object_id() {
        let filter = Filter {
            namespace_id: Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap(),
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
            namespace_id: Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap(),
            object_id: Some("namespace".to_owned()),
            key: None,
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:namespace"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_filter_with_only_namespace_id() {
        let filter = Filter {
            namespace_id: Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap(),
            object_id: None,
            key: None,
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71"}"#)
                .unwrap()
        );
    }

    #[test]
    fn deserialize_empty_filter() {
        let res = serde_json::from_str::<Request>(r#"{"fq":""}"#);

        assert!(res.is_err());

        let err = res.unwrap_err();
        assert_eq!(
            format!("{}", err),
            "missing field `namespace_id` at line 1 column 9"
        );
    }
}
