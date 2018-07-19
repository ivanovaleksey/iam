use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use jsonrpc;
use serde_json;
use uuid::Uuid;

use abac::models::AbacPolicy;
use abac::schema::abac_policy;
use abac::types::AbacAttribute;

use iam::models::{identity::PrimaryKey, Account, Namespace};
use iam::schema::{account, identity};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, IAM_NAMESPACE_ID,
    NETOLOGY_ACCOUNT_ID,
};

lazy_static! {
    static ref FOXFORD_USER_ID: Uuid = Uuid::new_v4();
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

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_client {
    use super::*;

    #[test]
    fn can_create_record_first_time() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();

        if let Ok(resp) = serde_json::from_slice::<jsonrpc::Success>(&body) {
            use iam::rpc::identity::create::Response;

            let identity: Response = serde_json::from_value(resp.result).unwrap();

            let pk = build_pk();
            assert_eq!(identity.id.provider, pk.provider);
            assert_eq!(identity.id.label, pk.label);
            assert_eq!(identity.id.uid, pk.uid);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn), Ok(1));

                let created_account = account::table
                    .find(identity.data.account_id)
                    .get_result::<Account>(&conn)
                    .unwrap();

                assert!(created_account.disabled_at.is_none());
            }

            let req_json = json!({
                "jsonrpc": "2.0",
                "method": "authorize",
                "params": [{
                    "namespace_ids": [*IAM_NAMESPACE_ID],
                    "subject": [
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "uri",
                            "value": format!("account/{}", identity.data.account_id),
                        }
                    ],
                    "object": [
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "uri",
                            "value": format!("identity/{}", identity.id),
                        }
                    ],
                    "action": [
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "operation",
                            "value": "read",
                        },
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "operation",
                            "value": "update",
                        },
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "operation",
                            "value": "delete",
                        }
                    ],
                }],
                "id": "qwerty",
            });
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&req_json).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_json = r#"{
                "jsonrpc": "2.0",
                "result": true,
                "id": "qwerty"
            }"#;
            assert_eq!(body, shared::strip_json(resp_json));

            let req_json = json!({
                "jsonrpc": "2.0",
                "method": "authorize",
                "params": [{
                    "namespace_ids": [*IAM_NAMESPACE_ID],
                    "subject": [
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "uri",
                            "value": format!("account/{}", *FOXFORD_ACCOUNT_ID),
                        }
                    ],
                    "object": [
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "uri",
                            "value": format!("identity/{}", identity.id),
                        }
                    ],
                    "action": [
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "operation",
                            "value": "read",
                        },
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "operation",
                            "value": "update",
                        },
                        {
                            "namespace_id": *IAM_NAMESPACE_ID,
                            "key": "operation",
                            "value": "delete",
                        }
                    ],
                }],
                "id": "qwerty",
            });
            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&req_json).unwrap(),
                Some(*IAM_ACCOUNT_ID),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            let resp_json = r#"{
                "jsonrpc": "2.0",
                "result": true,
                "id": "qwerty"
            }"#;
            assert_eq!(body, shared::strip_json(resp_json));
        } else {
            panic!("{:?}", body);
        }
    }

    #[test]
    fn cannot_create_record_second_time() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);

            let user_account = diesel::insert_into(account::table)
                .values(account::id.eq(Uuid::new_v4()))
                .get_result::<Account>(&conn)
                .unwrap();

            diesel::insert_into(identity::table)
                .values((
                    identity::provider.eq(*FOXFORD_NAMESPACE_ID),
                    identity::label.eq("oauth2"),
                    identity::uid.eq(FOXFORD_USER_ID.to_string()),
                    identity::account_id.eq(user_account.id),
                    identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
                ))
                .execute(&conn)
                .unwrap();
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_json = r#"{
            "jsonrpc": "2.0",
            "error": {
                "code": 422,
                "message": "Identity already exists"
            },
            "id": "qwerty"
        }"#;
        assert_eq!(body, shared::strip_json(resp_json));
    }

    #[test]
    fn cannot_create_alien_record() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*NETOLOGY_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn), Ok(0));
        }
    }

    #[test]
    fn can_create_alien_record_when_permission_granted() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);

            diesel::insert_into(abac_policy::table)
                .values(AbacPolicy {
                    subject: vec![AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "uri".to_owned(),
                        value: format!("account/{}", *NETOLOGY_ACCOUNT_ID),
                    }],
                    object: vec![
                        AbacAttribute {
                            namespace_id: *IAM_NAMESPACE_ID,
                            key: "uri".to_owned(),
                            value: format!("namespace/{}", *FOXFORD_NAMESPACE_ID),
                        },
                        AbacAttribute {
                            namespace_id: *IAM_NAMESPACE_ID,
                            key: "type".to_owned(),
                            value: "identity".to_owned(),
                        },
                    ],
                    action: vec![AbacAttribute {
                        namespace_id: *IAM_NAMESPACE_ID,
                        key: "operation".to_owned(),
                        value: "create".to_owned(),
                    }],
                    namespace_id: *IAM_NAMESPACE_ID,
                })
                .execute(&conn)
                .unwrap();
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*NETOLOGY_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();

        if let Ok(_) = serde_json::from_slice::<jsonrpc::Success>(&body) {
            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn), Ok(1));
            }
        } else {
            panic!("{:?}", body);
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
        shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::FORBIDDEN);

    {
        let conn = get_conn!(pool);
        assert_eq!(find_record(&conn), Ok(0));
    }
}

fn build_request() -> serde_json::Value {
    let payload = build_pk();
    json!({
        "jsonrpc": "2.0",
        "method": "identity.create",
        "params": [{
            "id": payload
        }],
        "id": "qwerty"
    })
}

fn build_pk() -> PrimaryKey {
    PrimaryKey {
        provider: *FOXFORD_NAMESPACE_ID,
        label: "oauth2".to_owned(),
        uid: FOXFORD_USER_ID.to_string(),
    }
}

fn find_record(conn: &PgConnection) -> diesel::QueryResult<usize> {
    let pk = build_pk();
    identity::table.find(pk.as_tuple()).execute(conn)
}
