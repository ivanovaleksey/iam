# Revoke

Revokes the old refresh token and issues a new one.

*NOTE: the operation isn't allowed for disabled accounts*

**URI**

```
POST /accounts/${KEY}/revoke
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`

**Response**

Name           | Type   | Default    | Description
-------------- | ------ | ---------- | ------------------
refresh\_token | string | _required_ | Used to refresh the access token, never expires

**Example**

```bash
curl -fsSL \
    -XPOST ${ENDPOINT}/accounts/me/revoke \
    -H"Authorization: Bearer ${REFRESH_TOKEN}" \
    | jq '.'
 
{
  "refresh_token": "eyJhbGci..."
}
```