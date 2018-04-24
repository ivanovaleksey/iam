extern crate actix_web;
extern crate iam;

use actix_web::{test::TestServer, HttpMessage};

fn build_server() -> TestServer {
    use std::env;

    TestServer::build_with_state::<_, iam::AppState>(|| {
        let database_url = env::var("DATABASE_URL").unwrap();
        iam::build_app_state(database_url)
    }).start(|app| {
        app.resource("/", |r| r.h(iam::call));
    })
}

#[test]
fn test_ping() {
    let mut srv = build_server();
    let req = srv.get()
        .body(r#"{"jsonrpc":"2.0","method":"ping","params":[],"id":"qwerty"}"#)
        .unwrap();

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, r#"{"jsonrpc":"2.0","result":"pong","id":"qwerty"}"#);
}
