pub use self::response::{FORBIDDEN, NOT_FOUND};

mod response {
    use shared;

    lazy_static! {
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