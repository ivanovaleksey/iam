# Disable

Only IAM administrator can disable an account.

### Method

```
account.disable
```

### Params

Name  | Type   | Default    | Description
----- | ------ | ---------- | ------------------
id    | uuid   | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "account.disable",
    "params": [{
        "id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420"
    }],
    "id": "qwerty"
}
```

#### Response

```json
{
    "jsonrpc": "2.0",
    "result": {
        "id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
        "data": {
            "disabled_at": "2018-07-20T19:40:00Z"
        }
    },
    "id": "qwerty"
}
```
