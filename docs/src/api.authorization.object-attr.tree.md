# List

### Method

```
abac_object_attr.tree
```

### Params

Name   | Type   | Default    | Description
-------| ------ | ---------- | ------------------
filter | object | _required_ | -
limit  | int    | see config | -
offset | int    | 0          | -

#### Filter

Name      | Type           | Default    | Description
--------- | -------------- | ---------- | ------------------
direction | string         | _required_ | inbound | outbound
attribute | abac_attribute | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_object_attr.tree",
    "params": [{
        "filter": {
            "direction": "outbound",
            "attribute": {
                "namespace_id": "c9040393-d4b5-4d0e-80d2-9081cf1c42b9",
                "key": "uri",
                "value": "webinar/1"
            }
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
            "namespace_id": "c9040393-d4b5-4d0e-80d2-9081cf1c42b9",
            "key": "type",
            "value": "webinar"
        },
        {
            "namespace_id": "c9040393-d4b5-4d0e-80d2-9081cf1c42b9",
            "key": "kind",
            "value": "math"
        }
    ],
    "id": "qwerty"
}
```
