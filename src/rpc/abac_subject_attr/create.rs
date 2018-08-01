use abac::{models::AbacSubject, AbacAttribute};
use futures::{future, Future};

use rpc;

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub inbound: AbacAttribute,
    pub outbound: AbacAttribute,
}

#[derive(Debug, Serialize)]
pub struct Response {
    inbound: AbacAttribute,
    outbound: AbacAttribute,
}

impl From<AbacSubject> for Response {
    fn from(subject: AbacSubject) -> Self {
        Response {
            inbound: subject.inbound,
            outbound: subject.outbound,
        }
    }
}

pub fn call(meta: rpc::Meta, req: Request) -> impl Future<Item = Response, Error = rpc::Error> {
    use abac_attribute::{CollectionKind, OperationKind};
    use actors::db;
    use rpc::authorize_collection;
    use uuid::Uuid;

    let Request { inbound, outbound } = req.clone();

    let inbound_ns_id = inbound.namespace_id;
    let outbound_ns_id = outbound.namespace_id;

    let collection = CollectionKind::AbacSubject;
    let operation = OperationKind::Create;

    future::result(rpc::forbid_anonymous(meta.subject))
        .and_then({
            let db = meta.db.clone().unwrap();
            move |subject_id| {
                authorize_collection(&db, inbound_ns_id, subject_id, collection, operation).or_else(
                    move |_| {
                        authorize_collection(&db, outbound_ns_id, subject_id, collection, operation)
                            .and_then(move |_| {
                                if inbound.key == "uri" {
                                    let mut parts = inbound.value.splitn(2, '/');
                                    if let (Some("account"), Some(uuid)) =
                                        (parts.next(), parts.next())
                                    {
                                        return Uuid::parse_str(uuid)
                                            .map_err(|_| rpc::error::Error::Forbidden);
                                    }
                                }
                                Err(rpc::error::Error::Forbidden)
                            })
                            .and_then({
                                move |account_id| {
                                    let msg =
                                        db::identity::select::Select::ByAccountIdAndProvider {
                                            account_id,
                                            provider: outbound_ns_id,
                                        };
                                    db.send(msg).from_err().and_then(|res| {
                                        if res?.is_empty() {
                                            Err(rpc::error::Error::Forbidden)
                                        } else {
                                            Ok(())
                                        }
                                    })
                                }
                            })
                    },
                )
            }
        })
        .and_then({
            let db = meta.db.unwrap();
            move |_| {
                let msg = db::abac_subject_attr::insert::Insert::from(req);
                db.send(msg).from_err().and_then(|res| {
                    debug!("abac subject insert res: {:?}", res);
                    Ok(Response::from(res?))
                })
            }
        })
}
