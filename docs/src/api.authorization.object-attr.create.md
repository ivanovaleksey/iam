# Create

### Method

```
abac_object_attr.create
```

### Params

Name         | Type   | Default    | Description
------------ | ------ | ---------- | ------------------
namespace_id | uuid   | _required_ | -
object_id    | string | _required_ | -
key          | string | _required_ | -
value        | string | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_object_attr.create",
    "params": [{
        "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "object_id": "room",
        "key": "type",
        "value": "room"
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
        "object_id": "room",
        "key": "type",
        "value": "room"
    },
    "id": "qwerty"
}
```
