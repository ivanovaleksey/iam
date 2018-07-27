# List

### Method

```
abac_policy.list
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

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_policy.list",
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
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "subject": [
                {
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "key": "uri",
                    "value": "account/f4e05c37-43a1-45aa-a8f8-fd656337cbc5"
                }
            ],
            "object": [
                {
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "key": "uri",
                    "value": "account/f4e05c37-43a1-45aa-a8f8-fd656337cbc5"
                }
            ],
            "action": [
                {
                    "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                    "key": "operation",
                    "value": "any"
                }
            ]
        }
    ],
    "id": "qwerty"
}
```
