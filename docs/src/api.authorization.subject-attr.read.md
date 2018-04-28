# Read

### Method

```
abac_subject_attr.read
```

### Params

Name         | Type   | Default    | Description
------------ | ------ | ---------- | ------------------
namespace_id | uuid   | _required_ | -
subject_id   | uuid   | _required_ | -
key          | string | _required_ | -
value        | string | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_subject_attr.read",
    "params": [{
        "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "subject_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
        "key": "role",
        "value": "client"
    }],
    "id": "qwerty"
}
```

#### Response

```json
{
    "jsonrpc": "2.0",
    "result": {
        "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "subject_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
        "key": "role",
        "value": "client"
    },
    "id": "qwerty"
}
```
