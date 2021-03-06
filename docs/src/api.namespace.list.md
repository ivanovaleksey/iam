# List

### Method

```
namespace.list
```

### Params

Name   | Type   | Default    | Description
------ | ------ | ---------- | ------------------
filter | object | _required_ | -
limit  | int    | see config | -
offset | int    | 0          | -

#### Filter

Name        | Type   | Default    | Description
----------- | ------ | ---------- | ------------------
account_id  | uuid   | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "namespace.list",
    "params": [{
        "filter": {
            "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420" 
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
            "id": "ed9eda41-bbae-44ba-83e0-1dd12b0f75c0",
            "data": {
                "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
                "label": "foxford.ru",
                "created_at": "2018-05-30T08:40:00Z"
            }
        }
    ],
    "id": "qwerty"
}
```
