# Create

### Description

As [described](api.identity.create.html#description) using `identity.create` generates new _account & identity_ pair.
Sequential calling `identity.create` & `identity.delete` would result in multiple accounts that might be not linked to any identities.

To avoid that it is not allowed to remove the last identity (an error would be returned).
This case should be considered as removing the whole account, and thus `account.delete` should be used instead.  

### Method

```
identity.delete
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
    "method": "identity.delete",
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
