use diesel;
use diesel::prelude::*;
use uuid::Uuid;

mod create;
mod delete;
mod list;
mod read;

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
