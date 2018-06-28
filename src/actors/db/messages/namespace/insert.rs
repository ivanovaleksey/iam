use abac::{models::AbacObject, schema::abac_object, types::AbacAttribute};
use actix::prelude::*;
use diesel::{self, prelude::*};
use uuid::Uuid;

use actors::DbExecutor;
use models::{Namespace, NewNamespace};
use rpc::error::Result;
use rpc::namespace::create;
use settings;

#[derive(Debug)]
pub struct Insert {
    pub label: String,
    pub account_id: Uuid,
    pub enabled: bool,
}

impl Message for Insert {
    type Result = Result<Namespace>;
}

impl Handler<Insert> for DbExecutor {
    type Result = Result<Namespace>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        call(conn, msg)
    }
}

impl From<create::Request> for Insert {
    fn from(req: create::Request) -> Self {
        Insert {
            label: req.label,
            account_id: req.account_id,
            enabled: req.enabled,
        }
    }
}

fn call(conn: &PgConnection, msg: Insert) -> Result<Namespace> {
    use schema::namespace;

    conn.transaction::<_, _, _>(|| {
        let changeset = NewNamespace::from(msg);
        let namespace = diesel::insert_into(namespace::table)
            .values(changeset)
            .get_result::<Namespace>(conn)?;

        let iam_namespace_id = settings::iam_namespace_id();

        let mut objects = Vec::with_capacity(6);

        objects.push(AbacObject {
            inbound: AbacAttribute {
                namespace_id: iam_namespace_id,
                key: "uri".to_owned(),
                value: format!("namespace/{}", namespace.id),
            },
            outbound: AbacAttribute {
                namespace_id: iam_namespace_id,
                key: "uri".to_owned(),
                value: format!("account/{}", namespace.account_id),
            },
        });

        objects.push(AbacObject {
            inbound: AbacAttribute {
                namespace_id: iam_namespace_id,
                key: "uri".to_owned(),
                value: format!("namespace/{}", namespace.id),
            },
            outbound: AbacAttribute {
                namespace_id: iam_namespace_id,
                key: "type".to_owned(),
                value: "namespace".to_owned(),
            },
        });

        ["abac_subject", "abac_object", "abac_action", "abac_policy"]
            .iter()
            .for_each(|collection| {
                objects.push(AbacObject {
                    inbound: AbacAttribute {
                        namespace_id: namespace.id,
                        key: "type".to_owned(),
                        value: collection.to_string(),
                    },
                    outbound: AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "uri".to_owned(),
                        value: format!("namespace/{}", namespace.id),
                    },
                });
            });

        diesel::insert_into(abac_object::table)
            .values(objects)
            .execute(conn)?;

        Ok(namespace)
    })
}
