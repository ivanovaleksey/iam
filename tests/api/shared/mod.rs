use actix_web::{self, http, test::TestServer};
use diesel;
use diesel::prelude::*;
use iam;
use uuid::Uuid;

pub struct Server {
    pub srv: TestServer,
    pub pool: iam::DbPool,
}

pub fn build_server() -> Server {
    use std::env;

    init();

    let database_url = env::var("DATABASE_URL").unwrap();
    let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(database_url);

    let pool = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to build pool");

    let pool1 = pool.clone();
    let srv =
        TestServer::build_with_state(move || iam::build_app_state(pool1.clone())).start(|app| {
            app.resource("/", |r| r.post().h(iam::call));
        });

    Server { srv, pool }
}

pub fn build_rpc_request(srv: &TestServer, json: String) -> actix_web::client::ClientRequest {
    srv.post()
        .header(http::header::AUTHORIZATION, "Bearer eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIyNWEwYzM2Ny03NTZhLTQyZTEtYWM1YS1lN2EyYjZiNjQ0MjAifQ==.MEYCIQDEUAevIcmf-MK7dPZpUPoPxemOTKZZeUYC7NbGDsI-9gIhALfQKFCoc761wS7CcIy0nDa54-QhiIAGeW1ObWv_GxDz")
        .content_type("application/json")
        .body(json)
        .unwrap()
}

pub fn build_anonymous_request(srv: &TestServer, json: String) -> actix_web::client::ClientRequest {
    srv.post()
        .content_type("application/json")
        .body(json)
        .unwrap()
}

fn init() {
    use env_logger;

    let _ = env_logger::try_init();
    iam::settings::init().expect("Failed to initialize settings");
}

pub fn strip_json(json: &str) -> String {
    json.replace('\n', "").replace(' ', "")
}

pub fn grant_namespace_ownership(conn: &PgConnection, namespace_id: Uuid, account_id: Uuid) {
    use iam::models::*;
    use iam::schema::*;

    diesel::insert_into(abac_subject_attr::table)
        .values(NewAbacSubjectAttr {
            namespace_id: namespace_id,
            subject_id: account_id,
            key: "owner:namespace".to_owned(),
            value: namespace_id.to_string(),
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_object_attr::table)
        .values(NewAbacObjectAttr {
            namespace_id: namespace_id,
            object_id: format!("namespace.{}", namespace_id),
            key: "belongs_to:namespace".to_owned(),
            value: namespace_id.to_string(),
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_action_attr::table)
        .values(NewAbacActionAttr {
            namespace_id: namespace_id,
            action_id: "execute".to_owned(),
            key: "access".to_owned(),
            value: "*".to_owned(),
        })
        .execute(conn)
        .unwrap();

    diesel::insert_into(abac_policy::table)
        .values(NewAbacPolicy {
            namespace_id: namespace_id,
            subject_namespace_id: namespace_id,
            subject_key: "owner:namespace".to_owned(),
            subject_value: namespace_id.to_string(),
            object_namespace_id: namespace_id,
            object_key: "belongs_to:namespace".to_owned(),
            object_value: namespace_id.to_string(),
            action_namespace_id: namespace_id,
            action_key: "access".to_owned(),
            action_value: "*".to_owned(),
            not_before: None,
            expired_at: None,
        })
        .execute(conn)
        .unwrap();
}
