# Read

### Method

```
abac_policy.read
```

### Params

Name                 | Type      | Default    | Description
-------------------- | --------- | ---------- | -----------
namespace_id         | uuid      | _required_ | –
subject_namespace_id | uuid      | _required_ | –
subject_key          | string    | _required_ | –
subject_value        | string    | _required_ | –
object_namespace_id  | uuid      | _required_ | –
object_key           | string    | _required_ | –
object_value         | string    | _required_ | –
action_namespace_id  | uuid      | _required_ | –
action_key           | string    | _required_ | –
action_value         | string    | _required_ | –

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_policy.read",
    "params": [{
        "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "subject_key": "role",
        "subject_value": "admin",
        "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "object_key": "type",
        "object_value": "namespace",
        "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "action_key": "action",
        "action_value": "*"
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
        "subject_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "subject_key": "role",
        "subject_value": "admin",
        "object_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "object_key": "type",
        "object_value": "namespace",
        "action_namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "action_key": "action",
        "action_value": "*",
        "issued_at": "2018-05-29T07:15:00",
        "not_before": null,
        "expired_at": null
    },
    "id": "qwerty"
}
```
