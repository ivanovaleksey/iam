# Delete

Removes the account.

*NOTE: the operation is only allowed for admins (members of 'admin' predefined group)*

**URI**

```
DELETE /accounts/${KEY}
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`

**Response**

Removed account

**Example**

```bash
curl -fsSL \
    -XDELETE ${ENDPOINT}/accounts/me \
    -H"Authorization: Bearer ${ACCESS_TOKEN}" \
    | jq '.'
 
{
  "id": "9074b6aa-a980-44e9-8973-29501900aa79"
}
```