# List

### Method

```
abac_action_attr.list
```

### Params

Name         | Type   | Default    | Description
------------ | ------ | ---------- | ------------------
fq           | string | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_action_attr.list",
    "params": [{
        "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND action_id:create AND key:access"
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
            "action_id": "create",
            "key": "access",
            "value": "*"
        }
    ],
    "id": "qwerty"
}
```
