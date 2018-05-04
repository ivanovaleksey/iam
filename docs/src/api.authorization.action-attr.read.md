# Read

### Method

```
abac_action_attr.read
```

### Params

Name         | Type   | Default    | Description
------------ | ------ | ---------- | ------------------
namespace_id | uuid   | _required_ | -
action_id    | string | _required_ | -
key          | string | _required_ | -
value        | string | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_action_attr.read",
    "params": [{
        "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "action_id": "create",
        "key": "access",
        "value": "*"
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
        "action_id": "create",
        "key": "access",
        "value": "*"
    },
    "id": "qwerty"
}
```
