use actix_web::HttpMessage;
use chrono::NaiveDate;
use diesel::{self, prelude::*};
use serde_json;
use uuid::Uuid;

use iam::models::{Account, Identity, Namespace};
use iam::schema::identity;

use shared::db::{create_account, create_namespace, create_operations, AccountKind, NamespaceKind};
use shared::{
    self, FOXFORD_ACCOUNT_ID, FOXFORD_NAMESPACE_ID, IAM_ACCOUNT_ID, NETOLOGY_NAMESPACE_ID,
};

lazy_static! {
    static ref FOXFORD_USER_1_ID: Uuid = Uuid::new_v4();
    static ref FOXFORD_USER_2_ID: Uuid = Uuid::new_v4();
    static ref NETOLOGY_USER_ID: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID_1: Uuid = Uuid::new_v4();
    static ref USER_ACCOUNT_ID_2: Uuid = Uuid::new_v4();
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

    create_records(conn);

    (
        (iam_account, iam_namespace),
        (foxford_account, foxford_namespace),
    )
}

mod with_admin {
    use super::*;

    #[test]
    fn without_filter() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(None, None);
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:01Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_1_ID"
                },
                {
                    "account_id": "USER_ACCOUNT_ID_2",
                    "created_at": "2018-06-02T08:40:02Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_2_ID"
                },
                {
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:03Z",
                    "label": "oauth2",
                    "provider": "NETOLOGY_NAMESPACE_ID",
                    "uid": "NETOLOGY_USER_ID"
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
            .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
            .replace("NETOLOGY_NAMESPACE_ID", &NETOLOGY_NAMESPACE_ID.to_string())
            .replace("NETOLOGY_USER_ID", &NETOLOGY_USER_ID.to_string())
            .replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string())
            .replace("USER_ACCOUNT_ID_2", &USER_ACCOUNT_ID_2.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_filter_by_provider() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:01Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_1_ID"
                },
                {
                    "account_id": "USER_ACCOUNT_ID_2",
                    "created_at": "2018-06-02T08:40:02Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_2_ID"
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
            .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
            .replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string())
            .replace("USER_ACCOUNT_ID_2", &USER_ACCOUNT_ID_2.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_filter_by_provider_and_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_ACCOUNT_ID_1));
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:01Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_1_ID"
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
            .replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_filter_by_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(None, Some(*USER_ACCOUNT_ID_2));
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*IAM_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "account_id": "USER_ACCOUNT_ID_2",
                    "created_at": "2018-06-02T08:40:02Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_2_ID"
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
            .replace("USER_ACCOUNT_ID_2", &USER_ACCOUNT_ID_2.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }
}

mod with_client {
    use super::*;

    #[test]
    fn without_filter() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(None, None);
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn with_filter_by_own_provider() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
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
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:01Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_1_ID"
                },
                {
                    "account_id": "USER_ACCOUNT_ID_2",
                    "created_at": "2018-06-02T08:40:02Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_2_ID"
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
            .replace("FOXFORD_USER_2_ID", &FOXFORD_USER_2_ID.to_string())
            .replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string())
            .replace("USER_ACCOUNT_ID_2", &USER_ACCOUNT_ID_2.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_filter_by_alien_provider() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(Some(*NETOLOGY_NAMESPACE_ID), None);
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn with_filter_by_provider_and_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_ACCOUNT_ID_1));
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
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:01Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_1_ID"
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
            .replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_filter_by_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(None, Some(*USER_ACCOUNT_ID_1));
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*FOXFORD_ACCOUNT_ID),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

mod with_user {
    use super::*;

    #[test]
    fn without_filter() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(None, None);
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn with_filter_by_provider() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }

    #[test]
    fn with_filter_by_provider_and_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), Some(*USER_ACCOUNT_ID_1));
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:01Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_1_ID"
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
            .replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_filter_by_own_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(None, Some(*USER_ACCOUNT_ID_1));
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        let resp_template = r#"{
            "jsonrpc": "2.0",
            "result": [
                {
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:01Z",
                    "label": "oauth2",
                    "provider": "FOXFORD_NAMESPACE_ID",
                    "uid": "FOXFORD_USER_1_ID"
                },
                {
                    "account_id": "USER_ACCOUNT_ID_1",
                    "created_at": "2018-06-02T08:40:03Z",
                    "label": "oauth2",
                    "provider": "NETOLOGY_NAMESPACE_ID",
                    "uid": "NETOLOGY_USER_ID"
                }
            ],
            "id": "qwerty"
        }"#;
        let resp_json = resp_template
            .replace("FOXFORD_NAMESPACE_ID", &FOXFORD_NAMESPACE_ID.to_string())
            .replace("NETOLOGY_NAMESPACE_ID", &NETOLOGY_NAMESPACE_ID.to_string())
            .replace("FOXFORD_USER_1_ID", &FOXFORD_USER_1_ID.to_string())
            .replace("NETOLOGY_USER_ID", &NETOLOGY_USER_ID.to_string())
            .replace("USER_ACCOUNT_ID_1", &USER_ACCOUNT_ID_1.to_string());
        assert_eq!(body, shared::strip_json(&resp_json));
    }

    #[test]
    fn with_filter_by_alien_account() {
        let shared::Server { mut srv, pool } = shared::build_server();

        {
            let conn = get_conn!(pool);
            let _ = before_each_1(&conn);
        }

        let payload = build_request(None, Some(*USER_ACCOUNT_ID_2));
        let req = shared::build_auth_request(
            &srv,
            serde_json::to_string(&payload).unwrap(),
            Some(*USER_ACCOUNT_ID_1),
        );
        let resp = srv.execute(req.send()).unwrap();
        let body = srv.execute(resp.body()).unwrap();
        assert_eq!(body, *shared::api::FORBIDDEN);
    }
}

#[test]
fn anonymous_cannot_list_identities() {
    let shared::Server { mut srv, pool } = shared::build_server();

    {
        let conn = get_conn!(pool);
        let _ = before_each_1(&conn);
    }

    let payload = build_request(Some(*FOXFORD_NAMESPACE_ID), None);
    let req = shared::build_anonymous_request(&srv, serde_json::to_string(&payload).unwrap());
    let resp = srv.execute(req.send()).unwrap();
    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, *shared::api::FORBIDDEN);
}

fn build_request(provider: Option<Uuid>, account_id: Option<Uuid>) -> serde_json::Value {
    let mut filter = json!({});

    if let Some(provider) = provider {
        filter["provider"] = serde_json::to_value(provider).unwrap();
    }
    if let Some(account_id) = account_id {
        filter["account_id"] = serde_json::to_value(account_id).unwrap();
    }

    json!({
        "jsonrpc": "2.0",
        "method": "identity.list",
        "params": [{
            "filter": filter
        }],
        "id": "qwerty"
    })
}

fn create_records(conn: &PgConnection) {
    use iam::actors::db;

    let user_account_1 = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_1));
    let user_account_2 = create_account(conn, AccountKind::Other(*USER_ACCOUNT_ID_2));

    let identities = vec![
        (
            identity::provider.eq(*FOXFORD_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_1_ID.to_string()),
            identity::account_id.eq(user_account_1.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 1)),
        ),
        (
            identity::provider.eq(*FOXFORD_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(FOXFORD_USER_2_ID.to_string()),
            identity::account_id.eq(user_account_2.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 2)),
        ),
        (
            identity::provider.eq(*NETOLOGY_NAMESPACE_ID),
            identity::label.eq("oauth2"),
            identity::uid.eq(NETOLOGY_USER_ID.to_string()),
            identity::account_id.eq(user_account_1.id),
            identity::created_at.eq(NaiveDate::from_ymd(2018, 6, 2).and_hms(8, 40, 3)),
        ),
    ];

    for identity in &identities {
        let identity = diesel::insert_into(identity::table)
            .values(identity)
            .get_result::<Identity>(conn)
            .unwrap();

        db::identity::insert::insert_identity_links(conn, &identity).unwrap();
    }
}
