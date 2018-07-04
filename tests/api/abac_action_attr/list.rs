use actix_web::HttpMessage;
use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::models::{AbacAction, AbacObject};
use abac::schema::{abac_action, abac_object};
use abac::types::AbacAttribute;

use iam::models::{Account, Namespace};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, NETOLOGY_ACCOUNT_ID, NETOLOGY_NAMESPACE_ID,
};

#[must_use]
fn before_each_1(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    let foxford_account = create_account(conn, AccountKind::Foxford);
    let foxford_namespace = create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

    let netology_account = create_account(conn, AccountKind::Netology);
    let netology_namespace = create_namespace(conn, NamespaceKind::Netology(netology_account.id));

    create_records(conn);

    diesel::insert_into(abac_object::table)
        .values(vec![
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: foxford_namespace.id,
                    key: "type".to_owned(),
                    value: "abac_action".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", foxford_namespace.id),
                },
            },
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: netology_namespace.id,
                    key: "type".to_owned(),
                    value: "abac_action".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", netology_namespace.id),
                },
            },
        ])
        .execute(conn)
        .unwrap();

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_client {
    use super::*;

    #[test]
    fn can_list_own_records_1() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(vec![*FOXFORD_NAMESPACE_ID])).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "inbound": {
                        "key": "operation",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "create"
                    },
                    "outbound": {
                        "key": "operation",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "any"
                    }
                },
                {
                    "inbound": {
                        "key": "operation",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "read"
                    },
                    "outbound": {
                        "key": "operation",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "any"
                    }
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json =
            resp_template.replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn can_list_own_records_2() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(vec![*NETOLOGY_NAMESPACE_ID])).unwrap(),
            Some(*NETOLOGY_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "inbound": {
                        "key": "operation",
                        "namespace_id": "NETOLOGY_NAMESPACE_ID",
                        "value": "create"
                    },
                    "outbound": {
                        "key": "operation",
                        "namespace_id": "NETOLOGY_NAMESPACE_ID",
                        "value": "any"
                    }
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json =
            resp_template.replace("NETOLOGY_NAMESPACE_ID", &NETOLOGY_NAMESPACE_ID.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn cannot_list_alien_records() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(vec![
                *FOXFORD_NAMESPACE_ID,
                *NETOLOGY_NAMESPACE_ID,
            ])).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

#[test]
fn anonymous_cannot_list_records() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let req = shared::build_anonymous_request(
        &srv,
        serde_json::to_string(&build_request(vec![*FOXFORD_NAMESPACE_ID])).unwrap(),
    );
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::FORBIDDEN);
}

fn build_request(ids: Vec<Uuid>) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "method": "abac_action_attr.list",
        "params": [{
            "filter": {
                "namespace_ids": ids
            }
        }],
        "id": "qwerty"
    })
}

fn create_records(conn: &PgConnection) {
    diesel::insert_into(abac_action::table)
        .values(vec![
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "operation".to_owned(),
                    value: "create".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "operation".to_owned(),
                    value: "read".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
            AbacAction {
                inbound: AbacAttribute {
                    namespace_id: *NETOLOGY_NAMESPACE_ID,
                    key: "operation".to_owned(),
                    value: "create".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: *NETOLOGY_NAMESPACE_ID,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                },
            },
        ])
        .execute(conn)
        .unwrap();
}
