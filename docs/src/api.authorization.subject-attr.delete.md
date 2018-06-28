# Delete

### Method

```
abac_subject_attr.delete
```

### Params

Name     | Type           | Default    | Description
-------- | -------------- | ---------- | ------------------
inbound  | abac_attribute | _required_ | -
outbound | abac_attribute | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_subject_attr.delete",
    "params": [{
        "inbound": {
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "uri",
            "value": "account/25a0c367-756a-42e1-ac5a-e7a2b6b64420"
        },
        "outbound": {
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "role",
            "value": "client"
        }
    }],
    "id": "qwerty"
}
```

#### Response

```json
{
    "jsonrpc": "2.0",
    "result": {
        "inbound": {
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "uri",
            "value": "account/25a0c367-756a-42e1-ac5a-e7a2b6b64420"
        },
        "outbound": {
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "role",
            "value": "client"
        }
    },
    "id": "qwerty"
}
```
