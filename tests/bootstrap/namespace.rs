use diesel;
use diesel::prelude::*;
use iam::models;
use iam::schema::*;

use bootstrap;
use helpers;

// Use case: Admin account creates foxford namespace
// "method": "namespace.create",
// "params": [{
//     "label": "foxford.ru",
//     "account_id": "$foxford_account_id",
//     "enabled": true
// }]
#[test]
fn admin_should_be_able_to_create() {
    // Known checks within `namespace.create` request are:
    //     - Should be able to `create` on `namespace` collection (considered to be granted via admin role)

    let conn = helpers::connection();

    let sql = include_str!("seeds.sql");
    diesel::sql_query(sql).execute(&conn).unwrap();

    let iam_ns = bootstrap::helpers::iam_namespace(&conn);

    let can_create = bootstrap::helpers::can(
        &conn,
        vec![iam_ns.id],
        iam_ns.account_id,
        "namespace",
        "create",
    );
    assert!(can_create);
}

#[test]
fn admin_should_be_able_to_read() {
    // Known checks within `namespace.read` request are:
    //     - Should be able to `read` on `namespace`

    let conn = helpers::connection();

    let sql = include_str!("seeds.sql");
    diesel::sql_query(sql).execute(&conn).unwrap();

    let iam_ns = bootstrap::helpers::iam_namespace(&conn);

    let can_read = bootstrap::helpers::can(
        &conn,
        vec![iam_ns.id],
        iam_ns.account_id,
        &format!("namespace.{}", iam_ns.id),
        "read",
    );
    assert!(can_read);
}

#[test]
fn admin_should_be_able_to_update() {
    // Known checks within `namespace.update` request are:
    //     - Should be able to `update` on `namespace`

    let conn = helpers::connection();

    let sql = include_str!("seeds.sql");
    diesel::sql_query(sql).execute(&conn).unwrap();

    let iam_ns = bootstrap::helpers::iam_namespace(&conn);

    let can_update = bootstrap::helpers::can(
        &conn,
        vec![iam_ns.id],
        iam_ns.account_id,
        &format!("namespace.{}", iam_ns.id),
        "update",
    );
    assert!(can_update);
}

#[test]
fn admin_should_be_able_to_delete() {
    // Known checks within `namespace.delete` request are:
    //     - Should be able to `delete` on `namespace`

    let conn = helpers::connection();

    let sql = include_str!("seeds.sql");
    diesel::sql_query(sql).execute(&conn).unwrap();

    let iam_ns = bootstrap::helpers::iam_namespace(&conn);

    let can_delete = bootstrap::helpers::can(
        &conn,
        vec![iam_ns.id],
        iam_ns.account_id,
        &format!("namespace.{}", iam_ns.id),
        "delete",
    );
    assert!(can_delete);
}

#[test]
fn admin_should_be_able_to_list() {
    // Known checks within `namespace.list` request are:
    //     - Should be able to `list` on `namespace`

    let conn = helpers::connection();

    let sql = include_str!("seeds.sql");
    diesel::sql_query(sql).execute(&conn).unwrap();

    let iam_ns = bootstrap::helpers::iam_namespace(&conn);

    let can_list = bootstrap::helpers::can(
        &conn,
        vec![iam_ns.id],
        iam_ns.account_id,
        "namespace",
        "list",
    );
    assert!(can_list);
}
