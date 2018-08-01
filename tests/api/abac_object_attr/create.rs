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
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, IAM_NAMESPACE_ID,
    NETOLOGY_ACCOUNT_ID,
};

lazy_static! {
    static ref WEBINAR_ID: Uuid = Uuid::new_v4();
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "uri",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "webinar/WEBINAR_ID"
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
            .replace("WEBINAR_ID", &WEBINAR_ID.to_string())
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string());

        shared::strip_json(&json)
    };
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

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_client {
    use super::*;

    #[test]
    fn can_create_own_record() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(None)).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *EXPECTED);

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn, None), Ok(1));
        }
    }

    #[test]
    fn cannot_create_alien_record() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
            let _ = create_account(&conn, AccountKind::Netology);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(None)).unwrap(),
            Some(*NETOLOGY_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn, None), Ok(0));
        }
    }

    #[test]
    fn can_create_alien_record_when_permission_granted() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let ((_iam_account, iam_namespace), (_foxford_account, foxford_namespace)) =
                before_each_1(&conn);

            let netology_account = create_account(&conn, AccountKind::Netology);

            diesel::insert_into(abac_policy::table)
                .values(NewAbacPolicy {
                    subject: vec![AbacAttribute::new(
                        iam_namespace.id,
                        UriKind::Account(netology_account.id),
                    )],
                    object: vec![
                        AbacAttribute::new(
                            iam_namespace.id,
                            UriKind::Namespace(foxford_namespace.id),
                        ),
                        AbacAttribute::new(iam_namespace.id, CollectionKind::AbacObject),
                    ],
                    action: vec![AbacAttribute::new(iam_namespace.id, OperationKind::Create)],
                    namespace_id: iam_namespace.id,
                })
                .execute(&conn)
                .unwrap();
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request(None)).unwrap(),
            Some(*NETOLOGY_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *EXPECTED);

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn, None), Ok(1));
        }
    }
}

mod when_storage_use_case {
    use super::*;

    lazy_static! {
        static ref STORAGE_ACCOUNT_ID: Uuid = Uuid::new_v4();
        static ref STORAGE_NAMESPACE_ID: Uuid = Uuid::new_v4();
        static ref SET_URI: String = "bucket_1/set_1".to_owned();
        static ref MOD_EXPECTED: String = {
            let template = r#"{
                "jsonrpc": "2.0",
                "result": {
                    "inbound": {
                        "key": "uri",
                        "namespace_id": "STORAGE_NAMESPACE_ID",
                        "value": "SET_URI"
                    },
                    "outbound": {
                        "key": "uri",
                        "namespace_id": "FOXFORD_NAMESPACE_ID",
                        "value": "group/1"
                    }
                },
                "id": "qwerty"
            }"#;

            let json = template
                .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
                .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
                .replace("STORAGE_NAMESPACE_ID", &STORAGE_NAMESPACE_ID.to_string())
                .replace("SET_URI", &SET_URI);

            shared::strip_json(&json)
        };
    }

    #[must_use]
    fn before_each_2(conn: &PgConnection) {
        let _ = before_each_1(conn);

        let storage_account = create_account(conn, AccountKind::Other(*STORAGE_ACCOUNT_ID));
        let storage_namespace = create_namespace(
            conn,
            NamespaceKind::Other {
                id: *STORAGE_NAMESPACE_ID,
                label: "storage.ng.services",
                account_id: storage_account.id,
            },
        );

        diesel::insert_into(abac_object::table)
            .values(vec![NewAbacObject {
                inbound: AbacAttribute {
                    namespace_id: storage_namespace.id,
                    key: "uri".to_owned(),
                    value: SET_URI.clone(),
                },
                outbound: AbacAttribute {
                    namespace_id: *IAM_NAMESPACE_ID,
                    key: "uri".to_owned(),
                    value: format!("namespace/{}", storage_namespace.id),
                },
            }])
            .execute(conn)
            .unwrap();
    }

    mod with_admin {
        use super::*;

        #[test]
        fn when_bucket_linked_to_foxford() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);

                diesel::insert_into(abac_object::table)
                    .values(NewAbacObject {
                        inbound: AbacAttribute {
                            namespace_id: *STORAGE_NAMESPACE_ID,
                            key: "uri".to_owned(),
                            value: SET_URI.clone(),
                        },
                        outbound: AbacAttribute {
                            namespace_id: *IAM_NAMESPACE_ID,
                            key: "uri".to_owned(),
                            value: format!("namespace/{}", *FOXFORD_NAMESPACE_ID),
                        },
                    })
                    .execute(&conn)
                    .unwrap();
            }

            let record = build_record();
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request(Some(&record))).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *MOD_EXPECTED);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn, Some(record)), Ok(1));
            }
        }

        #[test]
        fn when_bucket_not_linked_to_foxford() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let record = build_record();
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request(Some(&record))).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *MOD_EXPECTED);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn, Some(record)), Ok(1));
            }
        }
    }

    mod with_client {
        use super::*;

        #[test]
        fn when_bucket_linked_to_foxford() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);

                diesel::insert_into(abac_object::table)
                    .values(NewAbacObject {
                        inbound: AbacAttribute {
                            namespace_id: *STORAGE_NAMESPACE_ID,
                            key: "uri".to_owned(),
                            value: SET_URI.clone(),
                        },
                        outbound: AbacAttribute {
                            namespace_id: *IAM_NAMESPACE_ID,
                            key: "uri".to_owned(),
                            value: format!("namespace/{}", *FOXFORD_NAMESPACE_ID),
                        },
                    })
                    .execute(&conn)
                    .unwrap();
            }

            let record = build_record();
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request(Some(&record))).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *MOD_EXPECTED);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn, Some(record)), Ok(1));
            }
        }

        #[test]
        fn when_bucket_not_linked_to_foxford() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let record = build_record();
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request(Some(&record))).unwrap(),
                Some(*FOXFORD_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn, Some(record)), Ok(0));
            }
        }
    }

    fn build_record() -> NewAbacObject {
        NewAbacObject {
            inbound: AbacAttribute {
                namespace_id: *STORAGE_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: SET_URI.clone(),
            },
            outbound: AbacAttribute {
                namespace_id: *FOXFORD_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: "group/1".to_owned(),
            },
        }
    }
}

#[test]
fn anonymous_cannot_create_record() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let req =
        shared::build_anonymous_request(&srv, serde_json::to_string(&build_request(None)).unwrap());
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::FORBIDDEN);

    {
        let conn = get_conn!(pool);
        assert_eq!(find_record(&conn, None), Ok(0));
    }
}

fn build_request(record: Option<&NewAbacObject>) -> serde_json::Value {
    let default = build_record();

    json!({
        "jsonrpc": "2.0",
        "method": "abac_object_attr.create",
        "params": [record.or(Some(&default))],
        "id": "qwerty"
    })
}

fn build_record() -> NewAbacObject {
    NewAbacObject {
        inbound: AbacAttribute {
            namespace_id: *FOXFORD_NAMESPACE_ID,
            key: "uri".to_owned(),
            value: format!("webinar/{}", *WEBINAR_ID),
        },
        outbound: AbacAttribute {
            namespace_id: *IAM_NAMESPACE_ID,
            key: "uri".to_owned(),
            value: format!("namespace/{}", *FOXFORD_NAMESPACE_ID),
        },
    }
}

fn find_record(conn: &PgConnection, record: Option<NewAbacObject>) -> diesel::QueryResult<usize> {
    let record = record.or_else(|| Some(build_record())).unwrap();

    abac_object::table
        .find((record.inbound, record.outbound))
        .execute(conn)
}
