# Authorization

### Method

```
authorize
```

### Params

Name          | Type   | Default    | Description
------------- | ------ | ---------- | ------------------
namespace_ids | [uuid] | _required_ | -
subject       | string | _required_ | -
object        | string | _required_ | -
action        | string | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "authorize",
    "params": [{
        "namespace_ids": [
            "a6b56e6c-39a9-45c4-9720-9e288fd9bb3a"
        ],
        "subject": [
            {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "uri",
                "value": "account/25a0c367-756a-42e1-ac5a-e7a2b6b64420"
            }
        ],
        "object": [
            {
                "namespace_id": "a6b56e6c-39a9-45c4-9720-9e288fd9bb3a",
                "key": "uri",
                "value": "room/1"
            }
        ],
        "action": [
            {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "operation",
                "value": "create"
            }
        ]
    }],
    "id": "qwerty"
}
```

#### Response

```json
{
    "jsonrpc": "2.0",
    "result": true,
    "id": "qwerty"
}
```
