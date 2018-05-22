use actix_web::HttpMessage;

use shared;

#[test]
fn test_ping() {
    let shared::Server { mut srv, pool: _ } = shared::build_server();

    let req = srv
        .get()
        .body(r#"{"jsonrpc":"2.0","method":"ping","params":[],"id":"qwerty"}"#)
        .unwrap();

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    assert_eq!(body, r#"{"jsonrpc":"2.0","result":"pong","id":"qwerty"}"#);
}
