use futures::future::{self, Future};
use jsonrpc;
use uuid::Uuid;

use std::str;

use actors::db::identity;
use rpc;

pub type Request = rpc::ListRequest<Filter>;

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct Filter {
    pub provider: Option<Uuid>,
    pub account_id: Option<Uuid>,
}

impl str::FromStr for Filter {
    type Err = rpc::ListRequestFilterError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let mut filter = Filter::default();

        for part in s.split(" AND ") {
            let mut kv = part.splitn(2, ':');
            match (kv.next(), kv.next()) {
                (Some("provider"), Some(v)) => {
                    let uuid = Uuid::parse_str(v)?;
                    filter.provider = Some(uuid);
                }
                (Some("account_id"), Some(v)) => {
                    let uuid = Uuid::parse_str(v)?;
                    filter.account_id = Some(uuid);
                }
                _ => {}
            }
        }

        Ok(filter)
    }
}

pub type Response = rpc::ListResponse<rpc::identity::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject).and_then({
        let db = meta.db.unwrap();
        move |_subject_id| {
            let msg = identity::select::Select::from(req);
            db.send(msg)
                .map_err(|_| jsonrpc::Error::internal_error())
                .and_then(|res| {
                    debug!("identity select res: {:?}", res);

                    Ok(Response::from(res?))
                })
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json;

    #[test]
    fn deserialize_filter_with_all_fields() {
        let filter = Filter {
            provider: Some(Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap()),
            account_id: Some(Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(
                r#"{"fq":"provider:bab37008-3dc5-492c-af73-80c241241d71 AND account_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420"}"#
            ).unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_provider() {
        let filter = Filter {
            provider: None,
            account_id: Some(Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap()),
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(r#"{"fq":"account_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420"}"#)
                .unwrap()
        );
    }

    #[test]
    fn deserialize_filter_without_account_id() {
        let filter = Filter {
            provider: Some(Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap()),
            account_id: None,
        };
        let req = Request::new(filter);
        assert_eq!(
            req,
            serde_json::from_str(r#"{"fq":"provider:bab37008-3dc5-492c-af73-80c241241d71"}"#)
                .unwrap()
        );
    }

    #[test]
    fn deserialize_empty_filter() {
        let filter = Filter {
            provider: None,
            account_id: None,
        };
        let req = Request::new(filter);
        assert_eq!(req, serde_json::from_str(r#"{"fq":""}"#).unwrap());
    }
}
