# Read

### Method

```
abac_object_attr.read
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
    "method": "abac_object_attr.read",
    "params": [{
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
            "value": "room/5eb64c75-de2c-4a8a-b97b-dd3599e10450"
        },
        "outbound": {
            "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
            "key": "type",
            "value": "room"
        }
    },
    "id": "qwerty"
}
```
