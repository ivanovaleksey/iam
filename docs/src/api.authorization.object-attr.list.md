# List

### Method

```
abac_object_attr.list
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
    "method": "abac_object_attr.list",
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
                "key": "uri",
                "value": "room/5eb64c75-de2c-4a8a-b97b-dd3599e10450"
            },
            "outbound": {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "type",
                "value": "room"
            }
        }
    ],
    "id": "qwerty"
}
```
