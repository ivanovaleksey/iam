use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use uuid::Uuid;

use iam::models::*;
use iam::schema::*;

use shared;

pub fn create_iam_account(conn: &PgConnection) -> Account {
    diesel::insert_into(account::table)
        .values((
            account::id.eq(*shared::IAM_ACCOUNT_ID),
            account::enabled.eq(true),
        ))
        .get_result(conn)
        .unwrap()
}

pub fn create_iam_namespace(conn: &PgConnection, account_id: Uuid) -> Namespace {
    diesel::insert_into(namespace::table)
        .values((
            namespace::id.eq(*shared::IAM_NAMESPACE_ID),
            namespace::label.eq("iam.ng.services"),
            namespace::account_id.eq(account_id),
            namespace::enabled.eq(true),
            namespace::created_at.eq(NaiveDate::from_ymd(2018, 5, 30).and_hms(8, 40, 0)),
        ))
        .get_result(conn)
        .unwrap()
}

pub fn grant_namespace_ownership(conn: &PgConnection, namespace_id: Uuid, account_id: Uuid) {
    use chrono::{NaiveDate, NaiveDateTime};
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
            key: "action".to_owned(),
            value: "*".to_owned(),
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_policy::table)
        .values((
            abac_policy::namespace_id.eq(namespace_id),
            abac_policy::subject_namespace_id.eq(namespace_id),
            abac_policy::subject_key.eq("owner:namespace".to_owned()),
            abac_policy::subject_value.eq(namespace_id.to_string()),
            abac_policy::object_namespace_id.eq(namespace_id),
            abac_policy::object_key.eq("belongs_to:namespace".to_owned()),
            abac_policy::object_value.eq(namespace_id.to_string()),
            abac_policy::action_namespace_id.eq(namespace_id),
            abac_policy::action_key.eq("action".to_owned()),
            abac_policy::action_value.eq("*".to_owned()),
            abac_policy::created_at.eq(NaiveDate::from_ymd(2018, 5, 29).and_hms(7, 15, 0)),
            abac_policy::not_before.eq(None::<NaiveDateTime>),
            abac_policy::expired_at.eq(None::<NaiveDateTime>),
        ))
        .execute(conn)
        .unwrap();
}
