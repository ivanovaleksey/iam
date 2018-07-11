# Refresh

Issues a new access token of account. A previously issued refresh token is used to identify the subject of authentication.

*NOTE: the operation isn't allowed for disabled accounts*

**URI**

```
POST /accounts/${KEY}/refresh
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`

**Payload**

Name          | Type   | Default    | Description
------------- | ------ | ---------- | ------------------
expires\_in   | int    |        300 | Desired expiration time

**Response**

Name           | Type   | Default    | Description
-------------- | ------ | ---------- | ------------------
access\_token  | string | _required_ | Used for account identification.
expires\_in    | int    | _required_ | Expiration time of access token
token\_type    | string | _required_ | Always `Bearer`

**Example**

```bash
curl -fsSL \
    -XPOST ${ENDPOINT}/accounts/me/refresh \
    -H"Authorization: Bearer ${REFRESH_TOKEN}" \
    | jq '.'
 
{
  "access_token": "eyJhbGci...",
  "expires_in": 86400,
  "token_type": "Bearer"
}
```
