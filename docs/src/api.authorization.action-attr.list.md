# List

### Method

```
abac_action_attr.list
```

### Params

Name   | Type   | Default    | Description
-------| ------ | ---------- | ------------------
filter | string | _required_ | -

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
        }
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
