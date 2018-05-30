use actix_web::HttpMessage;

use shared;

#[test]
fn pong() {
    let shared::Server { mut srv, pool: _ } = shared::build_server();

    let req_json = r#"{
        "jsonrpc": "2.0",
        "method": "ping",
        "params": [],
        "id": "qwerty"
    }"#;
    let req = shared::build_anonymous_request(&srv, req_json.to_owned());

    let resp = srv.execute(req.send()).unwrap();
    assert!(resp.status().is_success());

    let body = srv.execute(resp.body()).unwrap();
    let resp_json = r#"{
        "jsonrpc": "2.0",
        "result": "pong",
        "id": "qwerty"
    }"#;
    assert_eq!(body, shared::strip_json(resp_json));
}
