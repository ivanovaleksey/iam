use abac::{models::AbacObject, types::AbacAttribute};
use futures::future::{self, Future};
use jsonrpc;

use actors::db::{abac_object_attr, authz::Authz};
use rpc;
use settings;

#[derive(Debug, Deserialize)]
pub struct Request {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

#[derive(Debug, Serialize)]
pub struct Response {
    inbound: AbacAttribute,
    outbound: AbacAttribute,
}

impl From<AbacObject> for Response {
    fn from(object: AbacObject) -> Self {
        Response {
            inbound: object.inbound,
            outbound: object.outbound,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = jsonrpc::Error> {
    let subject = rpc::forbid_anonymous(meta.subject);
    future::result(subject)
        .and_then({
            let db = meta.db.clone().unwrap();
            let namespace_id = req.outbound.namespace_id;
            move |subject_id| {
                let iam_namespace_id = settings::iam_namespace_id();

                let msg = Authz {
                    namespace_ids: vec![iam_namespace_id],
                    subject: vec![AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", subject_id),
                    }],
                    object: vec![AbacAttribute {
                        namespace_id,
                        key: "type".to_owned(),
                        value: "abac_object".to_owned(),
                    }],
                    action: vec![AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "operation".to_owned(),
                        value: "create".to_owned(),
                    }],
                };

                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(rpc::ensure_authorized)
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = abac_object_attr::insert::Insert::from(req);
                db.send(msg)
                    .map_err(|_| jsonrpc::Error::internal_error())
                    .and_then(|res| {
                        debug!("abac object insert res: {:?}", res);

                        Ok(Response::from(res?))
                    })
            }
        })
}
