# Create

### Method

```
abac_policy.create
```

### Params

Name         | Type             | Default    | Description
------------ | ---------------- | ---------- | -----------
namespace_id | uuid             | _required_ | –
subject      | [abac_attribute] | _required_ | –
object       | [abac_attribute] | _required_ | –
action       | [abac_attribute] | _required_ | –

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "abac_policy.create",
    "params": [{
        "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
        "subject": [
            {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "uri",
                "value": "account/f4e05c37-43a1-45aa-a8f8-fd656337cbc5"
            }
        ],
        "object": [
            {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "uri",
                "value": "account/f4e05c37-43a1-45aa-a8f8-fd656337cbc5"
            }
        ],
        "action": [
            {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "operation",
                "value": "any"
            }
        ]
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
        "subject": [
            {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "uri",
                "value": "account/f4e05c37-43a1-45aa-a8f8-fd656337cbc5"
            }
        ],
        "object": [
            {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "uri",
                "value": "account/f4e05c37-43a1-45aa-a8f8-fd656337cbc5"
            }
        ],
        "action": [
            {
                "namespace_id": "bab37008-3dc5-492c-af73-80c241241d71",
                "key": "operation",
                "value": "any"
            }
        ]
    },
    "id": "qwerty"
}
```
