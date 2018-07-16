use actix_web::HttpMessage;
use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use abac::models::{AbacPolicy, AbacSubject};
use abac::schema::{abac_policy, abac_subject};
use abac::types::AbacAttribute;

use iam::abac_attribute::{CollectionKind, OperationKind, UriKind};
use iam::models::{Account, Namespace, NewIdentity};
use iam::schema::identity;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_NAMESPACE_ID, NETOLOGY_ACCOUNT_ID,
    NETOLOGY_NAMESPACE_ID,
};

lazy_static! {
    static ref USER_ACCOUNT_ID: Uuid = Uuid::new_v4();
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "inbound": {
                    "key": "uri",
                    "namespace_id": "IAM_NAMESPACE_ID",
                    "value": "account/USER_ACCOUNT_ID"
                },
                "outbound": {
                    "key": "role",
                    "namespace_id": "FOXFORD_NAMESPACE_ID",
                    "value": "user"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("IAM_NAMESPACE_ID", &IAM_NAMESPACE_ID.to_string())
            .replace("USER_ACCOUNT_ID", &USER_ACCOUNT_ID.to_string())
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

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> ((Account, Namespace), (Account, Namespace)) {
        let ((iam_account, iam_namespace), (foxford_account, foxford_namespace)) =
            before_each_1(&conn);

        let user_account = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID));

        diesel::insert_into(identity::table)
            .values(NewIdentity {
                provider: foxford_namespace.id,
                label: "oauth2".to_owned(),
                uid: Uuid::new_v4().to_string(),
                account_id: user_account.id,
            })
            .execute(conn)
            .unwrap();

        (
            (iam_account, iam_namespace),
            (foxford_account, foxford_namespace),
        )
    }

    #[test]
    fn can_assign_role_to_own_user() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
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
    fn cannot_assign_role_to_alien_user() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
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
    fn cannot_assign_alien_role_to_own_user() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
            let _ = create_account(&conn, AccountKind::Netology);
        }

        let record = AbacSubject {
            inbound: AbacAttribute {
                namespace_id: *IAM_NAMESPACE_ID,
                key: "uri".to_owned(),
                value: format!("account/{}", *USER_ACCOUNT_ID),
            },
            outbound: AbacAttribute {
                namespace_id: *NETOLOGY_NAMESPACE_ID,
                key: "role".to_owned(),
                value: "user".to_owned(),
            },
        };

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

    #[test]
    fn can_assign_role_to_alien_user_when_permission_granted() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let ((_iam_account, iam_namespace), (_foxford_account, foxford_namespace)) =
                before_each_2(&conn);

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
                        AbacAttribute::new(iam_namespace.id, CollectionKind::AbacSubject),
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

fn build_request(record: Option<&AbacSubject>) -> serde_json::Value {
    let default = build_record();

    json!({
        "jsonrpc": "2.0",
        "method": "abac_subject_attr.create",
        "params": [record.or(Some(&default))],
        "id": "qwerty"
    })
}

fn build_record() -> AbacSubject {
    AbacSubject {
        inbound: AbacAttribute {
            namespace_id: *IAM_NAMESPACE_ID,
            key: "uri".to_owned(),
            value: format!("account/{}", *USER_ACCOUNT_ID),
        },
        outbound: AbacAttribute {
            namespace_id: *FOXFORD_NAMESPACE_ID,
            key: "role".to_owned(),
            value: "user".to_owned(),
        },
    }
}

fn find_record(conn: &PgConnection, record: Option<AbacSubject>) -> diesel::QueryResult<usize> {
    let record = record.or_else(|| Some(build_record())).unwrap();

    abac_subject::table
        .find((record.inbound, record.outbound))
        .execute(conn)
}
