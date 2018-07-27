pub use self::response::{BAD_REQUEST, FORBIDDEN, NOT_FOUND, UNAUTHORIZED};

mod response {
    use shared;

    lazy_static! {
        pub static ref BAD_REQUEST: String = {
            let json = r#"{
                "jsonrpc": "2.0",
                "error": {
                    "code": 400,
                    "message": "Bad request"
                },
                "id": "qwerty"
            }"#;
            shared::strip_json(json)
        };
        pub static ref UNAUTHORIZED: String = {
            let json = r#"{
                "jsonrpc": "2.0",
                "error": {
                    "code": 401,
                    "message": "Unauthorized"
                },
                "id": "qwerty"
            }"#;
            shared::strip_json(json)
        };
        pub static ref FORBIDDEN: String = {
            let json = r#"{
                "jsonrpc": "2.0",
                "error": {
                    "code": 403,
                    "message": "Forbidden"
                },
                "id": "qwerty"
            }"#;
            shared::strip_json(json)
        };
        pub static ref NOT_FOUND: String = {
            let json = r#"{
                "jsonrpc": "2.0",
                "error": {
                    "code": 404,
                    "message": "NotFound"
                },
                "id": "qwerty"
            }"#;
            shared::strip_json(json)
        };
    }
}
