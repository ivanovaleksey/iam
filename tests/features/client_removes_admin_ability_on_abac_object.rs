use actix_web::HttpMessage;
use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::models::{AbacObject, AbacPolicy, AbacSubject};
use abac::schema::{abac_object, abac_policy, abac_subject};
use abac::types::AbacAttribute;

use iam::models::{Account, Namespace};
use iam::schema::account;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, IAM_NAMESPACE_ID};

#[must_use]
fn before_each_1(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
    conn.begin_test_transaction()
        .expect("Failed to begin transaction");

    let iam_account = create_account(conn, AccountKind::Iam);
    let iam_namespace = create_namespace(conn, NamespaceKind::Iam(iam_account.id));

    create_operations(conn, iam_namespace.id);

    let foxford_account = create_account(conn, AccountKind::Foxford);
    let foxford_namespace = create_namespace(conn, NamespaceKind::Foxford(foxford_account.id));

    diesel::insert_into(abac_object::table)
        .values(vec![
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: foxford_namespace.id,
                    key: "type".to_owned(),
                    value: "abac_object".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", foxford_namespace.id),
                },
            },
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: foxford_namespace.id,
                    key: "type".to_owned(),
                    value: "abac_object".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "type".to_owned(),
                    value: "abac_object".to_owned(),
                },
            },
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "type".to_owned(),
                    value: "abac_object".to_owned(),
                },
                outbound: AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", iam_namespace.id),
                },
            },
        ])
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_policy::table)
        .values(vec![
            AbacPolicy {
                subject: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", iam_account.id),
                }],
                object: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", iam_account.id),
                }],
                action: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                }],
                namespace_id: iam_namespace.id,
            },
            AbacPolicy {
                subject: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", foxford_account.id),
                }],
                object: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "uri".to_owned(),
                    value: format!("account/{}", foxford_account.id),
                }],
                action: vec![AbacAttribute {
                    namespace_id: iam_namespace.id,
                    key: "operation".to_owned(),
                    value: "any".to_owned(),
                }],
                namespace_id: iam_namespace.id,
            },
        ])
        .execute(conn)
        .unwrap();

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

#[test]
fn test() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    // Initially client is able to create objects
    {
        let object = AbacObject {
            inbound: AbacAttribute {
                namespace_id: *FOXFORD_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: "webinar/1".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "type".to_owned(),
                value: "webinar".to_owned(),
            },
        };

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "abac_object_attr.create",
            "params": [object],
            "id": "qwerty"
        });

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "uri",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "webinar/1"
                },
                "outbound": {
                    "key": "type",
                    "namespace_id": "IAM_NAMESPACE_ID",
                    "value": "webinar"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

        assert_eq!(body, shared::strip_json(&json));
    }

    // Initially admin is able to create objects too
    {
        let object = AbacObject {
            inbound: AbacAttribute {
                namespace_id: *FOXFORD_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: "webinar/2".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "type".to_owned(),
                value: "webinar".to_owned(),
            },
        };

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "abac_object_attr.create",
            "params": [object],
            "id": "qwerty"
        });

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "uri",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "webinar/2"
                },
                "outbound": {
                    "key": "type",
                    "namespace_id": "IAM_NAMESPACE_ID",
                    "value": "webinar"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

        assert_eq!(body, shared::strip_json(&json));
    }

    // Then client delete admin's ability
    {
        let object = AbacObject {
            inbound: AbacAttribute {
                namespace_id: *FOXFORD_NAMESPACE_ID,
                key: "type".to_owned(),
                value: "abac_object".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "type".to_owned(),
                value: "abac_object".to_owned(),
            },
        };

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "abac_object_attr.delete",
            "params": [object],
            "id": "qwerty"
        });

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "type",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "abac_object"
                },
                "outbound": {
                    "key": "type",
                    "namespace_id": "IAM_NAMESPACE_ID",
                    "value": "abac_object"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

        assert_eq!(body, shared::strip_json(&json));



        let object = AbacObject {
            inbound: AbacAttribute {
                namespace_id: *FOXFORD_NAMESPACE_ID,
                key: "type".to_owned(),
                value: "abac_object".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: format!("namespace/{}", *FOXFORD_NAMESPACE_ID),
            },
        };

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "abac_object_attr.delete",
            "params": [object],
            "id": "qwerty"
        });

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "type",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "abac_object"
                },
                "outbound": {
                    "key": "uri",
                    "namespace_id": "IAM_NAMESPACE_ID",
                    "value": "namespace/FOXFORD_NAMESPACE_ID"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

        assert_eq!(body, shared::strip_json(&json));
    }

    // Now admin is unable to create objects
    {
        let object = AbacObject {
            inbound: AbacAttribute {
                namespace_id: *FOXFORD_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: "webinar/3".to_owned(),
            },
            outbound: AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "type".to_owned(),
                value: "webinar".to_owned(),
            },
        };

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "abac_object_attr.create",
            "params": [object],
            "id": "qwerty"
        });

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}
