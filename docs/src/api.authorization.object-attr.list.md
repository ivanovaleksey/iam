# List

### Method

```
abac_object_attr.list
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
    "method": "abac_object_attr.list",
    "params": [{
        "fq": "namespace_id:bab37008-3dc5-492c-af73-80c241241d71 AND object_id:room AND key:type"
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
            "object_id": "room",
            "key": "type",
            "value": "room"
        }
    ],
    "id": "qwerty"
}
```
