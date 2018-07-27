use actix_web::HttpMessage;
use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::prelude::*;
use abac::schema::*;

use iam::abac_attribute::{CollectionKind, OperationKind, UriKind};
use iam::models::{Account, Namespace};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_NAMESPACE_ID, NETOLOGY_ACCOUNT_ID,
    NETOLOGY_NAMESPACE_ID,
};

lazy_static! {
    static ref STORAGE_ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref STORAGE_NAMESPACE_ID: Uuid = Uuid::new_v4();
}

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
    let _netology_namespace = create_namespace(conn, NamespaceKind::Netology(netology_account.id));

    let storage_account = create_account(conn, AccountKind::Other(*STORAGE_ACCOUNT_ID));
    let _storage_namespace = create_namespace(
        conn,
        NamespaceKind::Other {
            id: *STORAGE_NAMESPACE_ID,
            label: "storage.ng.services",
            account_id: storage_account.id,
        },
    );

    create_records(conn);

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
                        "key": "uri",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "webinar/1"
                    },
                    "outbound": {
                        "key": "type",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "webinar"
                    }
                },
                {
                    "inbound": {
                        "key": "uri",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "webinar/2"
                    },
                    "outbound": {
                        "key": "type",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "webinar"
                    }
                },
                {
                    "inbound": {
                        "key": "uri",
                        "namespace_id": "STORAGE_NAMESPACE_ID",
                        "value": "bucket-1/set-1"
                    },
                    "outbound": {
                        "key": "uri",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "webinar/1"
                    }
                },
                {
                    "inbound": {
                        "key": "uri",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "webinar/1"
                    },
                    "outbound": {
                        "key": "kind",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "math"
                    }
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("STORAGE_NAMESPACE_ID", &STORAGE_NAMESPACE_ID.to_string());
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
                        "key": "uri",
                        "namespace_id": "NETOLOGY_NAMESPACE_ID",
                        "value": "webinar/1"
                    },
                    "outbound": {
                        "key": "type",
                        "namespace_id": "NETOLOGY_NAMESPACE_ID",
                        "value": "webinar"
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

    #[test]
    fn can_list_alien_records_when_permission_granted() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);

            diesel::insert_into(abac_policy::table)
                .values(NewAbacPolicy {
                    subject: vec![AbacAttribute::new(
                        *IAM_NAMESPACE_ID,
                        UriKind::Account(*FOXFORD_ACCOUNT_ID),
                    )],
                    object: vec![
                        AbacAttribute::new(
                            *IAM_NAMESPACE_ID,
                            UriKind::Namespace(*NETOLOGY_NAMESPACE_ID),
                        ),
                        AbacAttribute::new(*IAM_NAMESPACE_ID, CollectionKind::AbacObject),
                    ],
                    action: vec![AbacAttribute::new(*IAM_NAMESPACE_ID, OperationKind::List)],
                    namespace_id: *IAM_NAMESPACE_ID,
                })
                .execute(&conn)
                .unwrap();
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(vec![*NETOLOGY_NAMESPACE_ID])).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "inbound": {
                        "key": "uri",
                        "namespace_id": "NETOLOGY_NAMESPACE_ID",
                        "value": "webinar/1"
                    },
                    "outbound": {
                        "key": "type",
                        "namespace_id": "NETOLOGY_NAMESPACE_ID",
                        "value": "webinar"
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
    fn can_filter_by_namespace_both_inbound_and_outbound() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(vec![*STORAGE_NAMESPACE_ID])).unwrap(),
            Some(*STORAGE_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "inbound": {
                        "key": "uri",
                        "namespace_id": "STORAGE_NAMESPACE_ID",
                        "value": "bucket-1/set-1"
                    },
                    "outbound": {
                        "key": "uri",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "webinar/1"
                    }
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("STORAGE_NAMESPACE_ID", &STORAGE_NAMESPACE_ID.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn can_filter_by_key_both_inbound_and_outbound() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "abac_object_attr.list",
            "params": [{
                "filter": {
                    "namespace_ids": vec![*FOXFORD_NAMESPACE_ID],
                    "key": "kind",
                }
            }],
            "id": "qwerty"
        });

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "inbound": {
                        "key": "uri",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "webinar/1"
                    },
                    "outbound": {
                        "key": "kind",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "math"
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
    fn can_list_with_pagination() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        {
            let payload = json!({
                "jsonrpc": "2.0",
                "method": "abac_object_attr.list",
                "params": [{
                    "filter": {
                        "namespace_ids": vec![*FOXFORD_NAMESPACE_ID],
                    },
                    "limit": 1
                }],
                "id": "qwerty"
            });

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "inbound": {
                            "key": "uri",
                            "namespace_id": "FOXFORD_NAMESPACE_ID",
                            "value": "webinar/1"
                        },
                        "outbound": {
                            "key": "type",
                            "namespace_id": "FOXFORD_NAMESPACE_ID",
                            "value": "webinar"
                        }
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json =
                resp_template.replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        {
            let payload = json!({
                "jsonrpc": "2.0",
                "method": "abac_object_attr.list",
                "params": [{
                    "filter": {
                        "namespace_ids": vec![*FOXFORD_NAMESPACE_ID],
                    },
                    "limit": 2,
                    "offset": 1
                }],
                "id": "qwerty"
            });

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "inbound": {
                            "key": "uri",
                            "namespace_id": "FOXFORD_NAMESPACE_ID",
                            "value": "webinar/2"
                        },
                        "outbound": {
                            "key": "type",
                            "namespace_id": "FOXFORD_NAMESPACE_ID",
                            "value": "webinar"
                        }
                    },
                    {
                        "inbound": {
                            "key": "uri",
                            "namespace_id": "STORAGE_NAMESPACE_ID",
                            "value": "bucket-1/set-1"
                        },
                        "outbound": {
                            "key": "uri",
                            "namespace_id": "FOXFORD_NAMESPACE_ID",
                            "value": "webinar/1"
                        }
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json = resp_template
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("STORAGE_NAMESPACE_ID", &STORAGE_NAMESPACE_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        {
            let payload = json!({
                "jsonrpc": "2.0",
                "method": "abac_object_attr.list",
                "params": [{
                    "filter": {
                        "namespace_ids": vec![*FOXFORD_NAMESPACE_ID],
                    },
                    "offset": 3
                }],
                "id": "qwerty"
            });

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&payload).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_template = r#"{
                "jsonrpc": "2.0",
                "result": [
                    {
                        "inbound": {
                            "key": "uri",
                            "namespace_id": "FOXFORD_NAMESPACE_ID",
                            "value": "webinar/1"
                        },
                        "outbound": {
                            "key": "kind",
                            "namespace_id": "FOXFORD_NAMESPACE_ID",
                            "value": "math"
                        }
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json =
                resp_template.replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }
    }

    #[test]
    fn cannot_paginate_more_than_configured() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "abac_object_attr.list",
            "params": [{
                "filter": {
                    "namespace_ids": vec![*FOXFORD_NAMESPACE_ID],
                },
                "limit": 200
            }],
            "id": "qwerty"
        });

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::BAD_REQUEST);
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
        "method": "abac_object_attr.list",
        "params": [{
            "filter": {
                "namespace_ids": ids
            }
        }],
        "id": "qwerty"
    })
}

fn create_records(conn: &PgConnection) {
    diesel::insert_into(abac_object::table)
        .values(vec![
            NewAbacObject {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: "webinar/1".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "type".to_owned(),
                    value: "webinar".to_owned(),
                },
            },
            NewAbacObject {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: "webinar/2".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "type".to_owned(),
                    value: "webinar".to_owned(),
                },
            },
            NewAbacObject {
                inbound: AbacAttribute {
                    namespace_id: *NETOLOGY_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: "webinar/1".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: *NETOLOGY_NAMESPACE_ID,
                    key: "type".to_owned(),
                    value: "webinar".to_owned(),
                },
            },
            NewAbacObject {
                inbound: AbacAttribute {
                    namespace_id: *STORAGE_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: "bucket-1/set-1".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: "webinar/1".to_owned(),
                },
            },
            NewAbacObject {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: "webinar/1".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "kind".to_owned(),
                    value: "math".to_owned(),
                },
            },
        ])
        .execute(conn)
        .unwrap();
}
