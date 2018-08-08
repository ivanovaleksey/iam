use abac::AbacAttribute;
use actix::Addr;
use actix_web::{self, HttpMessage, HttpRequest, HttpResponse};
use diesel::QueryResult;
use futures::future::{self, Either, Future};
use jsonrpc::{self, MetaIoHandler, Metadata};
use serde_json;
use uuid::Uuid;

use std::fmt;

use abac_attribute::{CollectionKind, OperationKind, UriKind};
use actors::{db::authz::Authz, DbExecutor};
use authn;
use rpc::abac_action_attr::Rpc as AbacActionRpc;
use rpc::abac_object_attr::Rpc as AbacObjectRpc;
use rpc::abac_policy::Rpc as AbacPolicyRpc;
use rpc::abac_subject_attr::Rpc as AbacSubjectRpc;
use rpc::account::Rpc as AccountRpc;
use rpc::authz::Rpc as AuthRpc;
pub use rpc::error::{Error, Result};
use rpc::identity::Rpc as IdentityRpc;
use rpc::namespace::Rpc as NamespaceRpc;
use rpc::pagination::Pagination;
use rpc::ping::Rpc as PingRpc;
use AppState;

pub mod abac_action_attr;
pub mod abac_object_attr;
pub mod abac_policy;
pub mod abac_subject_attr;
pub mod account;
pub mod authz;
pub mod error;
pub mod identity;
pub mod namespace;
mod pagination;
mod ping;

// TODO: remove Default on new jsonrpc_core version
#[derive(Clone, Default)]
pub struct Meta {
    pub db: Option<Addr<DbExecutor>>,
    pub subject: Option<Uuid>,
}

impl fmt::Debug for Meta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Meta {{ subject: {:?} }}", self.subject)
    }
}

impl Metadata for Meta {}

pub type Server = MetaIoHandler<Meta>;

pub fn build_server() -> Server {
    let mut io = MetaIoHandler::default();

    let rpc = ping::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = authz::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = abac_subject_attr::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = abac_object_attr::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = abac_action_attr::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = abac_policy::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = namespace::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = identity::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    let rpc = account::RpcImpl {};
    io.extend_with(rpc.to_delegate());

    io
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<Id, Data> {
    pub id: Id,
    pub data: Data,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListRequest<T = ListRequestFilter> {
    pub filter: T,

    #[serde(flatten)]
    pub pagination: Pagination,
}

pub type TreeRequest = ListRequest<TreeRequestFilter>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListRequestFilter {
    pub namespace_ids: Vec<Uuid>,
    pub key: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct TreeRequestFilter {
    pub direction: DirectionKind,
    pub attribute: AbacAttribute,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DirectionKind {
    Inbound,
    Outbound,
}

#[derive(Debug, Serialize)]
pub struct ListResponse<T>(Vec<T>);

pub type TreeResponse = ListResponse<AbacAttribute>;

impl<T, I> From<Vec<I>> for ListResponse<T>
where
    T: From<I>,
{
    fn from(items: Vec<I>) -> Self {
        let items = items.into_iter().map(From::from).collect();
        ListResponse(items)
    }
}

pub fn ensure_authorized(res: QueryResult<bool>) -> Result<()> {
    if res? {
        Ok(())
    } else {
        Err(Error::Forbidden)
    }
}

pub fn forbid_anonymous(subject: Option<Uuid>) -> Result<Uuid> {
    subject.ok_or_else(|| Error::Forbidden)
}

pub fn index(
    req: HttpRequest<AppState>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    let mut meta = req.state().rpc_meta.clone();

    req.clone()
        .json()
        .from_err()
        .and_then(move |request: jsonrpc::Request| {
            use extract_authorization_header;

            let headers = req.headers();
            let res = match extract_authorization_header(&headers) {
                Ok(Some(value)) => {
                    let raw_token = authn::jwt::RawToken {
                        kind: authn::jwt::RawTokenKind::Iam,
                        value,
                    };
                    match authn::jwt::AccessToken::decode(&raw_token) {
                        Ok(token) => {
                            let validator = authn::jwt::Validator::default();
                            if validator.call(&token) {
                                meta.subject = Some(token.sub);
                                Ok(())
                            } else {
                                debug!("Invalid JWT");
                                Err(())
                            }
                        }
                        Err(e) => {
                            error!("{}", e);
                            Err(())
                        }
                    }
                }
                Ok(None) => Ok(()),
                Err(_) => Err(()),
            };

            if res.is_ok() {
                Either::A(
                    req.state()
                        .rpc_server
                        .handle_rpc_request(request, meta)
                        .map_err(|_| actix_web::error::ErrorInternalServerError("")),
                )
            } else {
                Either::B(
                    reject_request(&request)
                        .map_err(|_| actix_web::error::ErrorInternalServerError("")),
                )
            }
        })
        .then(|res| {
            res.or_else(|_| {
                let e = jsonrpc::Error::new(jsonrpc::ErrorCode::ParseError);
                let resp = jsonrpc::Response::from(e, Some(jsonrpc::Version::V2));
                Ok(Some(resp))
            })
        })
        .and_then(|resp| {
            if let Some(resp) = resp {
                let resp_str = serde_json::to_string(&resp).unwrap();
                Ok(HttpResponse::Ok().body(resp_str))
            } else {
                Ok(HttpResponse::Ok().into())
            }
        })
}

fn reject_request(
    request: &jsonrpc::Request,
) -> impl Future<Item = Option<jsonrpc::Response>, Error = ()> {
    match request {
        jsonrpc::Request::Single(call) => {
            let output = reject_call(call);
            let res = output.map(|o| o.map(jsonrpc::Response::Single));
            Either::A(res)
        }
        jsonrpc::Request::Batch(calls) => {
            let futures: Vec<_> = calls.iter().map(|c| reject_call(c)).collect();
            let res = future::join_all(futures).map(|outs| {
                let outs: Vec<_> = outs.into_iter().filter_map(|v| v).collect();
                Some(jsonrpc::Response::Batch(outs))
            });
            Either::B(res)
        }
    }
}

fn reject_call(call: &jsonrpc::Call) -> impl Future<Item = Option<jsonrpc::Output>, Error = ()> {
    let err = jsonrpc::Error {
        code: jsonrpc::ErrorCode::ServerError(401),
        message: "Unauthorized".to_owned(),
        data: None,
    };

    let output = match call {
        jsonrpc::Call::MethodCall(method) => {
            jsonrpc::Output::from(Err(err), method.id.clone(), method.jsonrpc)
        }
        jsonrpc::Call::Notification(notification) => {
            jsonrpc::Output::from(Err(err), jsonrpc::Id::Null, notification.jsonrpc)
        }
        jsonrpc::Call::Invalid(_id) => jsonrpc::Output::from(Err(err), jsonrpc::Id::Null, None),
    };

    future::ok(Some(output))
}

fn authorize_collection(
    db: &Addr<DbExecutor>,
    ns_id: Uuid,
    subject_id: Uuid,
    collection: CollectionKind,
    operation: OperationKind,
) -> impl Future<Item = (), Error = Error> {
    use settings;
    let iam_namespace_id = settings::iam_namespace_id();

    let subject = AbacAttribute::new(iam_namespace_id, UriKind::Account(subject_id));
    let ns = AbacAttribute::new(iam_namespace_id, UriKind::Namespace(ns_id));
    let collection = AbacAttribute::new(iam_namespace_id, collection);
    let action = AbacAttribute::new(iam_namespace_id, operation);

    let msg = Authz {
        namespace_ids: vec![iam_namespace_id],
        subject: vec![subject],
        object: vec![ns, collection],
        action: vec![action],
    };

    db.send(msg).from_err().and_then(ensure_authorized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_list_request_without_pagination() {
        let s = r#"{
            "filter": {
                "namespace_ids": ["bab37008-3dc5-492c-af73-80c241241d71"]
            }
        }"#;

        let req = serde_json::from_str::<ListRequest>(s);
        assert!(req.is_ok());

        let req = req.unwrap();
        let expected = Pagination {
            limit: 25,
            offset: 0,
        };
        assert_eq!(req.pagination, expected);
    }

    #[test]
    fn deserialize_list_request_with_pagination() {
        let s = r#"{
            "filter": {
                "namespace_ids": ["bab37008-3dc5-492c-af73-80c241241d71"]
            },
            "limit": 20,
            "offset": 3
        }"#;

        let req = serde_json::from_str::<ListRequest>(s);
        assert!(req.is_ok());

        let req = req.unwrap();
        let expected = Pagination {
            limit: 20,
            offset: 3,
        };
        assert_eq!(req.pagination, expected);
    }

    #[test]
    fn deserialize_list_request_with_limit() {
        let s = r#"{
            "filter": {
                "namespace_ids": ["bab37008-3dc5-492c-af73-80c241241d71"]
            },
            "limit": 20
        }"#;

        let req = serde_json::from_str::<ListRequest>(s);
        assert!(req.is_ok());

        let req = req.unwrap();
        let expected = Pagination {
            limit: 20,
            offset: 0,
        };
        assert_eq!(req.pagination, expected);
    }

    #[test]
    fn deserialize_list_request_with_offset() {
        let s = r#"{
            "filter": {
                "namespace_ids": ["bab37008-3dc5-492c-af73-80c241241d71"]
            },
            "offset": 3
        }"#;

        let req = serde_json::from_str::<ListRequest>(s);
        assert!(req.is_ok());

        let req = req.unwrap();
        let expected = Pagination {
            limit: 25,
            offset: 3,
        };
        assert_eq!(req.pagination, expected);
    }

    #[test]
    fn deserialize_tree_request_filter() {
        let s = r#"{
            "direction": "inbound",
            "attribute": {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "role",
                "value": "client"
            }
        }"#;

        let f = serde_json::from_str::<TreeRequestFilter>(s);
        assert!(f.is_ok());

        let f = f.unwrap();
        let expected = TreeRequestFilter {
            direction: DirectionKind::Inbound,
            attribute: AbacAttribute {
                namespace_id: Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap(),
                key: "role".to_owned(),
                value: "client".to_owned(),
            },
        };
        assert_eq!(f, expected);
    }
}
