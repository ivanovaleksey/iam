# Update

### Method

```
namespace.update
```

### Params

Name    | Type   | Default    | Description
------- | ------ | ---------- | ------------------
id      | uuid   | _required_ | -
label   | string | -          | -
enabled | bool   | -          | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "namespace.update",
    "params": [{
        "id": "ed9eda41-bbae-44ba-83e0-1dd12b0f75c0",
        "data": {
            "label": "foxford-new.ru"        
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
        "id": "ed9eda41-bbae-44ba-83e0-1dd12b0f75c0",
        "data": {
            "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
            "label": "foxford-new.ru",
            "created_at": "2018-05-30T08:40:00Z"
        }
    },
    "id": "qwerty"
}
```
