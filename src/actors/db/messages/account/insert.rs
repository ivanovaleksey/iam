use abac::{
    models::{AbacObject, AbacPolicy},
    schema::{abac_object, abac_policy},
    types::AbacAttribute,
};
use actix::prelude::*;
use diesel::{self, prelude::*};

use actors::DbExecutor;
use models::{Account, NewAccount};
use settings;

#[derive(Debug)]
pub struct Insert {
    pub enabled: bool,
}

impl Message for Insert {
    type Result = QueryResult<Account>;
}

impl Handler<Insert> for DbExecutor {
    type Result = QueryResult<Account>;

    fn handle(&mut self, msg: Insert, _ctx: &mut Self::Context) -> Self::Result {
        let conn = &self.0.get().unwrap();
        insert(conn, msg)
    }
}

pub fn insert(conn: &PgConnection, msg: Insert) -> QueryResult<Account> {
    use schema::account;

    let changeset = NewAccount::from(msg);

    conn.transaction::<_, _, _>(|| {
        let account = diesel::insert_into(account::table)
            .values(changeset)
            .get_result::<Account>(conn)?;

        let iam_namespace_id = settings::iam_namespace_id();

        diesel::insert_into(abac_policy::table)
            .values(AbacPolicy {
                subject: vec![AbacAttribute {
                    namespace_id: iam_namespace_id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", account.id),
                }],
                object: vec![AbacAttribute {
                    namespace_id: iam_namespace_id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", account.id),
                }],
                action: vec![AbacAttribute {
                    namespace_id: iam_namespace_id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                }],
                namespace_id: iam_namespace_id,
            })
            .execute(conn)?;

        diesel::insert_into(abac_object::table)
            .values(vec![
                AbacObject {
                    inbound: AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", account.id),
                    },
                    outbound: AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "type".to_owned(),
                        value: "account".to_owned(),
                    },
                },
                AbacObject {
                    inbound: AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", account.id),
                    },
                    outbound: AbacAttribute {
                        namespace_id: iam_namespace_id,
                        key: "uri".to_owned(),
                        value: format!("namespace/{}", iam_namespace_id),
                    },
                },
            ])
            .execute(conn)?;

        Ok(account)
    })
}
