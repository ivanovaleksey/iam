use abac::{
    models::{AbacObject, AbacPolicy, AbacSubject},
    schema::{abac_object, abac_policy, abac_subject}, types::AbacAttribute,
};
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::models::*;

use shared::{
    FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, IAM_NAMESPACE_ID,
    NETOLOGY_ACCOUNT_ID, NETOLOGY_NAMESPACE_ID,
};

pub enum AccountKind {
    Iam,
    Foxford,
    Netology,
    Other(Uuid),
}

pub enum NamespaceKind<'a> {
    Iam(Uuid),
    Foxford(Uuid),
    Netology(Uuid),
    Other {
        id: Uuid,
        label: &'a str,
        account_id: Uuid,
    },
}

pub fn create_account(conn: &PgConnection, kind: AccountKind) -> Account {
    use self::AccountKind::*;
    use iam::schema::account;

    let id = match kind {
        Iam => *IAM_ACCOUNT_ID,
        Foxford => *FOXFORD_ACCOUNT_ID,
        Netology => *NETOLOGY_ACCOUNT_ID,
        Other(id) => id,
    };

    let account = diesel::insert_into(account::table)
        .values((account::id.eq(id), account::enabled.eq(true)))
        .get_result::<Account>(conn)
        .unwrap();

    diesel::insert_into(abac_policy::table)
        .values(AbacPolicy {
            subject: vec![AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: format!("account/{}", account.id),
            }],
            object: vec![AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: format!("account/{}", account.id),
            }],
            action: vec![AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "operation".to_owned(),
                value: "any".to_owned(),
            }],
            namespace_id: *IAM_NAMESPACE_ID,
        })
        .execute(conn)
        .unwrap();

    match kind {
        Iam => {}
        Foxford | Netology => {
            diesel::insert_into(abac_subject::table)
                .values(AbacSubject {
                    inbound: AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("account/{}", account.id),
                    },
                    outbound: AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "role".to_owned(),
                        value: "client".to_owned(),
                    },
                })
                .execute(conn)
                .unwrap();

            link_account_to_iam(conn, account.id);
        }
        Other(_) => {
            link_account_to_iam(conn, account.id);
        }
    }

    account
}

pub fn create_namespace(conn: &PgConnection, kind: NamespaceKind) -> Namespace {
    use self::NamespaceKind::*;
    use iam::schema::namespace;

    let (id, label, account_id) = match kind {
        Iam(account_id) => (*IAM_NAMESPACE_ID, "iam.ng.services", account_id),
        Foxford(account_id) => (*FOXFORD_NAMESPACE_ID, "foxford.ru", account_id),
        Netology(account_id) => (*NETOLOGY_NAMESPACE_ID, "netology.ru", account_id),
        Other {
            id,
            label,
            account_id,
        } => (id, label, account_id),
    };

    let namespace = diesel::insert_into(namespace::table)
        .values((
            namespace::id.eq(id),
            namespace::label.eq(label),
            namespace::account_id.eq(account_id),
            namespace::enabled.eq(true),
            namespace::created_at.eq(NaiveDate::from_ymd(2018, 5, 30).and_hms(8, 40, 0)),
        ))
        .get_result::<Namespace>(conn)
        .unwrap();

    diesel::insert_into(abac_object::table)
        .values(vec![
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", namespace.id),
                },
                outbound: AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "type".to_owned(),
                    value: "namespace".to_owned(),
                },
            },
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", namespace.id),
                },
                outbound: AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("account/{}", namespace.account_id),
                },
            },
        ])
        .execute(conn)
        .unwrap();

    if let Iam(_) = kind {
        diesel::insert_into(abac_object::table)
            .values(AbacObject {
                inbound: AbacAttribute {
                    namespace_id: namespace.id,
                    key: "type".to_owned(),
                    value: "abac_object".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: namespace.id,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", namespace.id),
                },
            })
            .execute(conn)
            .unwrap();
    }

    namespace
}

pub fn create_operations(conn: &PgConnection, namespace_id: Uuid) {
    use abac::models::AbacAction;
    use abac::schema::abac_action;
    use abac::types::AbacAttribute;

    diesel::insert_into(abac_action::table)
        .values(vec![
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "create".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "read".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "update".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "delete".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "list".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
        ])
        .execute(conn)
        .unwrap();
}

fn link_account_to_iam(conn: &PgConnection, account_id: Uuid) {
    diesel::insert_into(abac_object::table)
        .values(AbacObject {
            inbound: AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: format!("account/{}", account_id),
            },
            outbound: AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: format!("namespace/{}", *IAM_NAMESPACE_ID),
            },
        })
        .execute(conn)
        .unwrap();
}
