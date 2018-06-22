use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::models::{AbacObject, AbacPolicy};
use abac::schema::{abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::models::{Account, Namespace};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID};

lazy_static! {
    static ref NETOLOGY_ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref NETOLOGY_NAMESPACE_ID: Uuid = Uuid::new_v4();
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

    let netology_account = create_account(conn, AccountKind::Other(*NETOLOGY_ACCOUNT_ID));
    let netology_namespace = create_namespace(
        conn,
        NamespaceKind::Other {
            id: *NETOLOGY_NAMESPACE_ID,
            label: "netology.ru",
            account_id: netology_account.id,
        },
    );

    create_records(conn);

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
                    namespace_id: netology_namespace.id,
                    key: "type".to_owned(),
                    value: "abac_object".to_owned(),
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

mod with_namespace_ownership {
    use super::*;
    use actix_web::HttpMessage;

    fn before_each_2(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
        let ((iam_account, iam_namespace), (foxford_account, foxford_namespace)) =
            before_each_1(conn);

        diesel::insert_into(abac_policy::table)
            .values(vec![
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
                AbacPolicy {
                    subject: vec![AbacAttribute {
                        namespace_id: iam_namespace.id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", *NETOLOGY_ACCOUNT_ID),
                    }],
                    object: vec![AbacAttribute {
                        namespace_id: iam_namespace.id,
                        key: "uri".to_owned(),
                        value: format!("account/{}", *NETOLOGY_ACCOUNT_ID),
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

    mod when_authorized_request {
        use super::*;

        #[test]
        fn when_all_namespace_ids_permitted_1() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
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
                    }
                ],
                "id": "qwerty"
            }"#;
            let resp_json =
                resp_template.replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());
            assert_eq!(body, shared::strip_json(&resp_json));
        }

        #[test]
        fn when_all_namespace_ids_permitted_2() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
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
        fn when_not_all_namespace_ids_permitted() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
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
    fn when_anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_anonymous_request(
            &srv,
            serde_json::to_string(&build_request(vec![*FOXFORD_NAMESPACE_ID])).unwrap(),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

mod without_namespace_ownership {
    use super::*;
    use actix_web::HttpMessage;

    fn before_each_2(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
        before_each_1(conn)
    }

    #[test]
    fn when_authorized_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(vec![*FOXFORD_NAMESPACE_ID])).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn when_anonymous_request() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_anonymous_request(
            &srv,
            serde_json::to_string(&build_request(vec![*FOXFORD_NAMESPACE_ID])).unwrap(),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
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
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("webinar/1"),
                },
                outbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "type".to_owned(),
                    value: "webinar".to_owned(),
                },
            },
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("webinar/2"),
                },
                outbound: AbacAttribute {
                    namespace_id: *FOXFORD_NAMESPACE_ID,
                    key: "type".to_owned(),
                    value: "webinar".to_owned(),
                },
            },
            AbacObject {
                inbound: AbacAttribute {
                    namespace_id: *NETOLOGY_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("webinar/1"),
                },
                outbound: AbacAttribute {
                    namespace_id: *NETOLOGY_NAMESPACE_ID,
                    key: "type".to_owned(),
                    value: "webinar".to_owned(),
                },
            },
        ])
        .execute(conn)
        .unwrap();
}
