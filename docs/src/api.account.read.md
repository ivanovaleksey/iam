# Read

### Method

```
account.read
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
    "method": "account.read",
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
        "id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420"
    },
    "id": "qwerty"
}
```
