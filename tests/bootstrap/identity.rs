use diesel;
use diesel::prelude::*;
use iam::models;
use iam::schema::*;

use bootstrap;
use helpers;

// "method": "identity.create",
// "params": [{
//     "provider": "$iam_ns_id",
//     "label": "trusted",
//     "uid": "foxford.ru",
//     "issuer_id": "$iam_account_id"
// }]
#[test]
fn client_should_be_able_to_create() {
    // Known checks within `identity.create` request are:
    //     - Should be able to `create` on `identity` collection
    //     - Should be able to `execute` on a given namespace (considered to be granted via namespace ownership)

    let conn = helpers::connection();

    let sql = include_str!("seeds.sql");
    diesel::sql_query(sql).execute(&conn).unwrap();

    let iam_ns = bootstrap::helpers::iam_namespace(&conn);

    let can_create = bootstrap::helpers::can(
        &conn,
        vec![iam_ns.id],
        iam_ns.account_id,
        "identity",
        "create",
    );
    assert!(!can_create);

    diesel::insert_into(abac_object_attr::table)
        .values(models::NewAbacObjectAttr {
            namespace_id: iam_ns.id,
            object_id: "identity".to_owned(),
            key: "type".to_owned(),
            value: "identity".to_owned(),
        })
        .execute(&conn)
        .unwrap();

    diesel::insert_into(abac_policy::table)
        .values(models::NewAbacPolicy {
            namespace_id: iam_ns.id,
            subject_namespace_id: iam_ns.id,
            subject_key: "role".to_owned(),
            subject_value: "client".to_owned(),
            object_namespace_id: iam_ns.id,
            object_key: "type".to_owned(),
            object_value: "identity".to_owned(),
            action_namespace_id: iam_ns.id,
            action_key: "action".to_owned(),
            action_value: "*".to_owned(),
            not_before: None,
            expired_at: None,
        })
        .execute(&conn)
        .unwrap();

    let can_create = bootstrap::helpers::can(
        &conn,
        vec![iam_ns.id],
        iam_ns.account_id,
        "identity",
        "create",
    );
    assert!(can_create);
}

// #[test]
// fn user_should_not_be_able_to_create() {}

// #[test]
// fn user_should_be_able_to_read() {}

// #[test]
// fn user_should_be_able_to_delete() {}
