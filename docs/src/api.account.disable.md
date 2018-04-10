# Disable

Disables account.

*NOTE: the operation is only allowed for admins (members of 'admin' predefined group)*

**URI**

```
DELETE /accounts/${KEY}/enabled
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`

**Example**

```bash
curl -fsSL \
    -XDELETE ${ENDPOINT}/accounts/9074b6aa-a980-44e9-8973-29501900aa79/disabled \
    -H"Authorization: Bearer ${ACCESS_TOKEN}" \
    | jq '.'
```