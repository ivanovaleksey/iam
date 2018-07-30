# List

### Method

```
abac_subject_attr.list
```

### Params

Name   | Type   | Default    | Description
-------| ------ | ---------- | ------------------
filter | object | _required_ | -

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
    "method": "abac_subject_attr.list",
    "params": [{
        "filter": {
            "namespace_ids": [
                "a6b56e6c-39a9-45c4-9720-9e288fd9bb3a"
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
                "key": "uri",
                "value": "account/25a0c367-756a-42e1-ac5a-e7a2b6b64420"
            },
            "outbound": {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "role",
                "value": "client"
            }
        }
    ],
    "id": "qwerty"
}
```
