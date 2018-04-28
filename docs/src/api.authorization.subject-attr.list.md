# List

### Method

```
abac_subject_attr.list
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
    "method": "abac_subject_attr.list",
    "params": [{
        "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND subject_id:25a0c367-756a-42e1-ac5a-e7a2b6b64420 AND key:role"
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
            "subject_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "key": "role",
            "value": "client"
        }
    ],
    "id": "qwerty"
}
```
