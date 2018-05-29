use futures::future::{self, Future};
use jsonrpc;
use serde_json;
use uuid::Uuid;

use std::str;

use actors::db::{abac_policy, authz::Authz};
use rpc;

pub type Request = rpc::ListRequest<Filter>;

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct Filter {
    pub namespace_id: Uuid,
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

pub type Response = rpc::ListResponse<rpc::abac_policy::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = meta.subject.ok_or(rpc::error::Error::Forbidden.into());
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.filter.0.namespace_id;
            move |subject_id| {
                let msg = Authz::execute_namespace_message(namespace_id, subject_id);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_policy::select::Select::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("abac policy select res: {:?}", res);
                        Ok(Response::from(res?))
                    })
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_filter_with_namespace_id() {
        let filter = Filter {
            namespace_id: Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap(),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(r#"{"fq":"namespace_id:bab37008-3dc5-492c-af73-80c241241d71"}"#)
                .unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_namespace_id() {
        let res = serde_json::from_str::<Request>(r#"{"fq":""}"#);

        assert!(res.is_err());

        let err = res.unwrap_err();
        assert_eq!(
            format!("{}", err),
            "missing field `namespace_id` at line 1 column 9"
        );
    }
}