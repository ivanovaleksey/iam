use actix_web::HttpMessage;
use diesel::{self, prelude::*};
use serde_json;

use abac::models::{AbacPolicy, NewAbacAction};
use abac::schema::{abac_action, abac_policy};
use abac::AbacAttribute;

use iam::abac_attribute::{CollectionKind, OperationKind, UriKind};
use iam::models::{Account, Namespace};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_NAMESPACE_ID, NETOLOGY_ACCOUNT_ID,
};

lazy_static! {
    static ref OPERATION: &'static str = "execute";
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "operation",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "OPERATION"
                },
                "outbound": {
                    "key": "operation",
                    "namespace_id": "IAM_NAMESPACE_ID",
                    "value": "any"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("OPERATION", &OPERATION.to_string())
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
            serde_json::to_string(&build_request()).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *EXPECTED);

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn), Ok(1));
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
            let ((_iam_account, iam_namespace), (_foxford_account, foxford_namespace)) =
                before_each_1(&conn);

            let netology_account = create_account(&conn, AccountKind::Netology);

            diesel::insert_into(abac_policy::table)
                .values(AbacPolicy {
                    subject: vec![AbacAttribute::new(
                        iam_namespace.id,
                        UriKind::Account(netology_account.id),
                    )],
                    object: vec![
                        AbacAttribute::new(
                            iam_namespace.id,
                            UriKind::Namespace(foxford_namespace.id),
                        ),
                        AbacAttribute::new(iam_namespace.id, CollectionKind::AbacAction),
                    ],
                    action: vec![AbacAttribute::new(iam_namespace.id, OperationKind::Create)],
                    namespace_id: iam_namespace.id,
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
        assert_eq!(body, *EXPECTED);

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn), Ok(1));
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
    let action = build_record();
    json!({
        "jsonrpc": "2.0",
        "method": "abac_action_attr.create",
        "params": [action],
        "id": "qwerty"
    })
}

fn build_record() -> NewAbacAction {
    NewAbacAction {
        inbound: AbacAttribute {
            namespace_id: *FOXFORD_NAMESPACE_ID,
            key: "operation".to_owned(),
            value: OPERATION.to_owned(),
        },
        outbound: AbacAttribute {
            namespace_id: *IAM_NAMESPACE_ID,
            key: "operation".to_owned(),
            value: "any".to_owned(),
        },
    }
}

fn find_record(conn: &PgConnection) -> diesel::QueryResult<usize> {
    let action = build_record();
    abac_action::table
        .find((action.inbound, action.outbound))
        .execute(conn)
}
