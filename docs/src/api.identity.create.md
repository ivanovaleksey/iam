# Create

### Description

An identity belongs to an account. We would like to avoid situations where an identity could be linked to a user account without user permission. For that reason `account_id` is intentionally omitted within payload.  

`(provider, label, uid)` triple is used to find existing identity. If an identity isn't found a new account is created. An identity is then linked to the created account. 
 
This endpoint is considered to be used only one time to generate initial _account & identity_ pair.  
Then a user can link new identities via Authentication API.

### Method

```
identity.create
```

### Params

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
provider  | uuid   | _required_ | -
label     | string | _required_ | -
uid       | string | _required_ | -

### Example

#### Request

```json
{
    "jsonrpc": "2.0",
    "method": "identity.create",
    "params": [{
        "provider": "bab37008-3dc5-492c-af73-80c241241d71",
        "label": "trusted",
        "uid": "foxford.ru"
    }],
    "id": "qwerty"
}
```

#### Response

```json
{
    "jsonrpc": "2.0",
    "result": {
        "provider": "bab37008-3dc5-492c-af73-80c241241d71",
        "label": "trusted",
        "uid": "foxford.ru",
        "account_id": "25a0c367-756a-42e1-ac5a-e7a2b6b64420",
        "created_at": "2018-06-02T08:40:00"
    },
    "id": "qwerty"
}
```
