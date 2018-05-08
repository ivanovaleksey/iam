# List

### Method

```
abac_policy.list
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
    "method": "abac_policy.list",
    "params": [{
        "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71"
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
            "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "subject_key": "role",
            "subject_value": "admin",
            "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "object_key": "type",
            "object_value": "namespace",
            "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "action_key": "action",
            "action_value": "*",
            "issued_at": "2018-05-29T07:15:00",
            "not_before": null,
            "expired_at": null
        }
    ],
    "id": "qwerty"
}
```
