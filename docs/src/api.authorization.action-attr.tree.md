# List

### Method

```
abac_action_attr.tree
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
    "method": "abac_action_attr.tree",
    "params": [{
        "filter": {
            "direction": "inbound",
            "attribute": {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "operation",
                "value": "create"
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
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "operation",
            "value": "any"
        }
    ],
    "id": "qwerty"
}
```
