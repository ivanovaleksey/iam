# Read

Returns the account.

**URI**

```
GET /accounts/${KEY}
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`

**Response**

Account

**Example**

```bash
curl -fsSL \
    -XPOST ${ENDPOINT}/accounts/me \
    -H"Authorization: Bearer ${ACCESS_TOKEN}" \
    | jq '.'
 
{
  "id": "9074b6aa-a980-44e9-8973-29501900aa79"
}
```