# Create

### Method

```
abac_action_attr.create
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
    "method": "abac_action_attr.create",
    "params": [{
        "inbound": {
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "operation",
            "value": "read"
        },
        "outbound": {
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "operation",
            "value": "any"
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
            "key": "operation",
            "value": "read"
        },
        "outbound": {
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "operation",
            "value": "any"
        }
    },
    "id": "qwerty"
}
```
