use futures::Future;
use jsonrpc;
use serde_json;
use uuid::Uuid;

use std::str;

use actors::db::namespace;
use rpc;

pub type Request = rpc::ListRequest<Filter>;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
pub struct Filter {
    pub account_id: Uuid,
}

impl str::FromStr for Filter {
    type Err = rpc::ListRequestFilterError;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        let mut filter = Filter::default();
        let mut is_account_present = false;

        for part in s.split(" AND ") {
            let mut kv = part.splitn(2, ':');
            match (kv.next(), kv.next()) {
                (Some("account_id"), Some(v)) => {
                    let uuid = Uuid::parse_str(v)?;
                    filter.account_id = uuid;
                    is_account_present = true;
                }
                _ => {}
            }
        }

        if !is_account_present {
            use serde::de::Error;
            return Err(serde_json::Error::missing_field("account_id"))?;
        }

        Ok(filter)
    }
}

pub type Response = rpc::ListResponse<rpc::namespace::read::Response>;

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let msg = namespace::select::Select::from(req);
    meta.db
        .unwrap()
        .send(msg)
        .map_err(|_| jsonrpc::Error::internal_error())
        .and_then(|res| {
            debug!("namespace select res: {:?}", res);
            match res {
                Ok(res) => Ok(Response::from(res)),
                Err(e) => Err(e.into()),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_filter_with_account_id() {
        let filter = Filter {
            account_id: Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap(),
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
        let res = serde_json::from_str::<Request>(r#"{"fq":""}"#);

        assert!(res.is_err());

        let err = res.unwrap_err();
        assert_eq!(
            format!("{}", err),
            "missing field `account_id` at line 1 column 9"
        );
    }
}
