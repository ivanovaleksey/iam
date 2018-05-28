use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::models::*;
use iam::schema::*;

pub fn create_iam_account(conn: &PgConnection) -> Account {
    diesel::insert_into(account::table)
        .values((
            account::id.eq(Uuid::parse_str("25a0c367-756a-42e1-ac5a-e7a2b6b64420").unwrap()),
            account::enabled.eq(true),
        ))
        .get_result(conn)
        .unwrap()
}

pub fn create_iam_namespace(conn: &PgConnection, account_id: Uuid) -> Namespace {
    diesel::insert_into(namespace::table)
        .values((
            namespace::id.eq(Uuid::parse_str("bab37008-3dc5-492c-af73-80c241241d71").unwrap()),
            namespace::label.eq("iam.ng.services"),
            namespace::account_id.eq(account_id),
            namespace::enabled.eq(true),
        ))
        .get_result(conn)
        .unwrap()
}

pub fn grant_namespace_ownership(conn: &PgConnection, namespace_id: Uuid, account_id: Uuid) {
    use iam::models::*;
    use iam::schema::*;

    diesel::insert_into(abac_subject_attr::table)
        .values(NewAbacSubjectAttr {
            namespace_id: namespace_id,
            subject_id: account_id,
            key: "owner:namespace".to_owned(),
            value: namespace_id.to_string(),
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_object_attr::table)
        .values(NewAbacObjectAttr {
            namespace_id: namespace_id,
            object_id: format!("namespace.{}", namespace_id),
            key: "belongs_to:namespace".to_owned(),
            value: namespace_id.to_string(),
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_action_attr::table)
        .values(NewAbacActionAttr {
            namespace_id: namespace_id,
            action_id: "execute".to_owned(),
            key: "access".to_owned(),
            value: "*".to_owned(),
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_policy::table)
        .values(NewAbacPolicy {
            namespace_id: namespace_id,
            subject_namespace_id: namespace_id,
            subject_key: "owner:namespace".to_owned(),
            subject_value: namespace_id.to_string(),
            object_namespace_id: namespace_id,
            object_key: "belongs_to:namespace".to_owned(),
            object_value: namespace_id.to_string(),
            action_namespace_id: namespace_id,
            action_key: "access".to_owned(),
            action_value: "*".to_owned(),
            not_before: None,
            expired_at: None,
        })
        .execute(conn)
        .unwrap();
}
