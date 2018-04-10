# List

Returns list of client's identities previously added to the account.

**URI**

```
GET /accounts/${KEY}/auth
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`

**Response**

List of client's identities.

**Example**

```bash
curl -fsSL \
    -XGET ${ENDPOINT}/accounts/me/auth \
    -H"Authorization: Bearer ${ACCESS_TOKEN}" \
    | jq '.'

[
  {
    "id": "123.oauth2.example.org"
  }
]
```