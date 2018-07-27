# List

### Method

```
abac_action_attr.list
```

### Params

Name   | Type   | Default    | Description
-------| ------ | ---------- | ------------------
filter | object | _required_ | -
limit  | int    | see config | -
offset | int    | 0          | -

#### Filter

Name          | Type   | Default    | Description
------------- | ------ | ---------- | ------------------
namespace_ids | [uuid] | _required_ | -
key           | string | -          | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_action_attr.list",
    "params": [{
        "filter": {
            "namespace_ids": [
                "bab37008-3dc5-492c-af73-80c241241d71"
            ]
        },
        "limit": 25,
        "offset": 0
    }],
    "id": "qwerty"
}
```

#### Response

```json
{
    "jsonrpc": "2.0",
    "result": [
        {
            "inbound": {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "operation",
                "value": "read"
            },
            "outbound": {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "operation",
                "value": "any"
            }
        }
    ],
    "id": "qwerty"
}
```
