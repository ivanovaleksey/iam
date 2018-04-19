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
            "bab37008-3dc5-492c-af73-80c241241d71"
        ],
        "subject": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
        "object": "room",
        "action": "create"
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
