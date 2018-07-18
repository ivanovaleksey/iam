use chrono::NaiveDate;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

use abac::schema::{abac_object, abac_policy};
use abac::types::AbacAttribute;

use iam::abac_attribute::UriKind;
use iam::models::{identity::PrimaryKey, Account, Identity, Namespace};
use iam::schema::{account, identity};

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_NAMESPACE_ID, NETOLOGY_ACCOUNT_ID,
    NETOLOGY_NAMESPACE_ID,
};

lazy_static! {
    static ref FOXFORD_USER_ID_1: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID_1: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID_2: Uuid = Uuid::new_v4();
    static ref EXPECTED: String = {
        let template = r#"{
            "jsonrpc": "2.0",
            "result": {
                "data": {
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:00Z"
                },
                "id": {
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_ID_1"
                }
            },
            "id": "qwerty"
        }"#;

        let json = template
            .replace("FOXFORD_USER_ID_1", &FOXFORD_USER_ID_1.to_string())
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string());

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

    let netology_account = create_account(conn, AccountKind::Netology);
    let _netology_namespace = create_namespace(conn, NamespaceKind::Netology(netology_account.id));

    let _user_account_2 = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_2));

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) -> Identity {
        let _ = before_each_1(conn);
        create_user_identity(conn)
    }

    mod with_client {
        use super::*;

        #[test]
        fn can_delete_last_identity() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
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
                assert_eq!(find_record(&conn), Ok(0));

                let found = account::table.find(*USER_ACCOUNT_ID_1).execute(&conn);
                assert_eq!(found, Ok(0));

                assert_eq!(identity_objects_count(&conn), Ok(0));
                assert_eq!(account_objects_count(&conn), Ok(0));
                assert_eq!(account_policies_count(&conn), Ok(0));
            }
        }

        #[test]
        fn can_delete_not_last_identity() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
                create_additional_user_identity(&conn);
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
                assert_eq!(find_record(&conn), Ok(0));

                let found = account::table.find(*USER_ACCOUNT_ID_1).execute(&conn);
                assert_eq!(found, Ok(1));

                assert_eq!(identity_objects_count(&conn), Ok(0));
                assert_eq!(account_objects_count(&conn), Ok(2));
                assert_eq!(account_policies_count(&conn), Ok(1));
            }
        }

        #[test]
        fn cannot_delete_alien_provider_identity() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
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
                assert_eq!(find_record(&conn), Ok(1));

                let found = account::table.find(*USER_ACCOUNT_ID_1).execute(&conn);
                assert_eq!(found, Ok(1));

                assert_eq!(identity_objects_count(&conn), Ok(3));
                assert_eq!(account_objects_count(&conn), Ok(2));
                assert_eq!(account_policies_count(&conn), Ok(1));
            }
        }
    }

    mod with_user {
        use super::*;

        #[test]
        fn can_delete_last_identity() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*USER_ACCOUNT_ID_1),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *EXPECTED);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn), Ok(0));

                let found = account::table.find(*USER_ACCOUNT_ID_1).execute(&conn);
                assert_eq!(found, Ok(0));

                assert_eq!(identity_objects_count(&conn), Ok(0));
                assert_eq!(account_objects_count(&conn), Ok(0));
                assert_eq!(account_policies_count(&conn), Ok(0));
            }
        }

        #[test]
        fn can_delete_not_last_identity() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
                create_additional_user_identity(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*USER_ACCOUNT_ID_1),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *EXPECTED);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn), Ok(0));

                let found = account::table.find(*USER_ACCOUNT_ID_1).execute(&conn);
                assert_eq!(found, Ok(1));

                assert_eq!(identity_objects_count(&conn), Ok(0));
                assert_eq!(account_objects_count(&conn), Ok(2));
                assert_eq!(account_policies_count(&conn), Ok(1));
            }
        }

        #[test]
        fn cannot_delete_alien_user_identity() {
            let shared::Server { mut srv, pool } = shared::build_server();

            {
                let conn = get_conn!(pool);
                let _ = before_each_2(&conn);
            }

            let req = shared::build_auth_request(
                &srv,
                serde_json::to_string(&build_request()).unwrap(),
                Some(*USER_ACCOUNT_ID_2),
            );
            let resp = srv.execute(req.send()).unwrap();
            let body = srv.execute(resp.body()).unwrap();
            assert_eq!(body, *shared::api::FORBIDDEN);

            {
                let conn = get_conn!(pool);
                assert_eq!(find_record(&conn), Ok(1));

                let found = account::table.find(*USER_ACCOUNT_ID_1).execute(&conn);
                assert_eq!(found, Ok(1));

                assert_eq!(identity_objects_count(&conn), Ok(3));
                assert_eq!(account_objects_count(&conn), Ok(2));
                assert_eq!(account_policies_count(&conn), Ok(1));
            }
        }
    }

    #[test]
    fn anonymous_cannot_delete_identity() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);

        {
            let conn = get_conn!(pool);
            assert_eq!(find_record(&conn), Ok(1));

            let found = account::table.find(*USER_ACCOUNT_ID_1).execute(&conn);
            assert_eq!(found, Ok(1));
        }
    }
}

mod without_existing_record {
    use super::*;
    use actix_web::HttpMessage;

    #[must_use]
    fn before_each_2(conn: &PgConnection) {
        let _ = before_each_1(conn);
    }

    #[test]
    fn client_cannot_delete_identity() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::NOT_FOUND);
    }

    #[test]
    fn user_cannot_delete_identity() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&build_request()).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn anonymous_cannot_delete_identity() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_2(&conn);
        }

        let req =
            shared::build_anonymous_request(&srv, serde_json::to_string(&build_request()).unwrap());
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

fn build_request() -> serde_json::Value {
    let payload = build_pk();
    json!({
        "jsonrpc": "2.0",
        "method": "identity.delete",
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
        uid: FOXFORD_USER_ID_1.to_string(),
    }
}

fn find_record(conn: &PgConnection) -> diesel::QueryResult<usize> {
    let pk = build_pk();
    identity::table.find(pk.as_tuple()).execute(conn)
}

fn create_user_identity(conn: &PgConnection) -> Identity {
    use iam::actors::db;

    let account = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_1));

    let identity = diesel::insert_into(identity::table)
        .values((
            identity::provider.eq(*FOXFORD_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_ID_1.to_string()),
            identity::account_id.eq(account.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
        ))
        .get_result::<Identity>(conn)
        .unwrap();

    db::identity::insert::insert_identity_links(conn, &identity).unwrap();

    identity
}

fn create_additional_user_identity(conn: &PgConnection) -> Identity {
    diesel::insert_into(identity::table)
        .values((
            identity::provider.eq(*NETOLOGY_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(Uuid::new_v4().to_string()),
            identity::account_id.eq(*USER_ACCOUNT_ID_1),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 0)),
        ))
        .get_result(conn)
        .unwrap()
}

fn identity_objects_count(conn: &PgConnection) -> diesel::QueryResult<usize> {
    let pk = PrimaryKey {
        provider: *FOXFORD_NAMESPACE_ID,
        label: "oauth2".to_owned(),
        uid: FOXFORD_USER_ID_1.to_string(),
    };

    abac_object::table
        .filter(
            abac_object::inbound.eq(AbacAttribute::new(*IAM_NAMESPACE_ID, UriKind::Identity(pk))),
        )
        .execute(conn)
}

fn account_objects_count(conn: &PgConnection) -> diesel::QueryResult<usize> {
    abac_object::table
        .filter(abac_object::inbound.eq(AbacAttribute::new(
            *IAM_NAMESPACE_ID,
            UriKind::Account(*USER_ACCOUNT_ID_1),
        )))
        .execute(conn)
}

fn account_policies_count(conn: &PgConnection) -> diesel::QueryResult<usize> {
    abac_policy::table
        .filter(abac_policy::subject.eq(vec![AbacAttribute::new(
            *IAM_NAMESPACE_ID,
            UriKind::Account(*USER_ACCOUNT_ID_1),
        )]))
        .execute(conn)
}
