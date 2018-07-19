# List

### Method

```
identity.list
```

### Params

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
filter | object | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "identity.list",
    "params": [{
        "filter": {
            "provider": "bab37008-3dc5-492c-af73-80c241241d71" 
        }
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
            "id": {
                "provider": "bab37008-3dc5-492c-af73-80c241241d71",
                "label": "trusted",
                "uid": "foxford.ru"
            },
            "data": {
                "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
                "created_at": "2018-06-02T08:40:00Z"
            }
        }
    ],
    "id": "qwerty"
}
```
