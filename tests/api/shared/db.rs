use abac::{models::AbacObject, schema::abac_object, AbacAttribute};
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
    use iam::actors::db;
    use iam::schema::account;

    let id = match kind {
        Iam => *IAM_ACCOUNT_ID,
        Foxford => *FOXFORD_ACCOUNT_ID,
        Netology => *NETOLOGY_ACCOUNT_ID,
        Other(id) => id,
    };

    let account = diesel::insert_into(account::table)
        .values(account::id.eq(id))
        .get_result::<Account>(conn)
        .unwrap();

    db::account::insert::insert_account_policies(conn, account.id).unwrap();

    match kind {
        Iam => {}
        _ => {
            db::account::insert::insert_account_links(conn, account.id).unwrap();
        }
    }

    account
}

pub fn create_namespace(conn: &PgConnection, kind: NamespaceKind) -> Namespace {
    use self::NamespaceKind::*;
    use iam::actors::db;
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
            namespace::created_at.eq(NaiveDate::from_ymd(2018, 5, 30).and_hms(8, 40, 0)),
        ))
        .get_result::<Namespace>(conn)
        .unwrap();

    db::namespace::insert::insert_namespace_links(conn, &namespace).unwrap();

    if let Iam(_) = kind {
        let objects = [
            "account",
            "namespace",
            "identity",
            "abac_subject",
            "abac_object",
            "abac_action",
            "abac_policy",
        ].iter()
            .map(|collection| AbacObject {
                inbound: AbacAttribute {
                    namespace_id: namespace.id,
                    key: "type".to_owned(),
                    value: collection.to_string(),
                },
                outbound: AbacAttribute {
                    namespace_id: namespace.id,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", namespace.id),
                },
            })
            .collect::<Vec<_>>();

        diesel::insert_into(abac_object::table)
            .values(objects)
            .execute(conn)
            .unwrap();
    }

    namespace
}

pub fn create_operations(conn: &PgConnection, namespace_id: Uuid) {
    use abac::models::AbacAction;
    use abac::schema::abac_action;
    use abac::AbacAttribute;

    let operations = ["create", "read", "update", "delete", "list"]
        .iter()
        .map(|operation| AbacAction {
            inbound: AbacAttribute {
                namespace_id,
                key: "operation".to_owned(),
                value: operation.to_string(),
            },
            outbound: AbacAttribute {
                namespace_id,
                key: "operation".to_owned(),
                value: "any".to_owned(),
            },
        })
        .collect::<Vec<_>>();

    diesel::insert_into(abac_action::table)
        .values(operations)
        .execute(conn)
        .unwrap();
}
