# Check if enabled

Returns **204 Success** status code if account is enabled, otherwise - **404 Not Found**.

*NOTE: the operation is only allowed for admins (members of 'admin' predefined group)*

**URI**

```
GET /accounts/${KEY}/enabled
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`

**Example**

```bash
curl -fsSL \
    -XGET ${ENDPOINT}/accounts/9074b6aa-a980-44e9-8973-29501900aa79/disabled \
    -H"Authorization: Bearer ${ACCESS_TOKEN}" \
    | jq '.'
```
