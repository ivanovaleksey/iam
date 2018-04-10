# Delete

Removes the client's identity.

*NOTE: the operation isn't allowed for disabled accounts*

**URI**

```
DELETE /accounts/${KEY}/auth/${IDENTITY}
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`
IDENTITY  | string | _required_ | Client's identity identifier

**Response**

Removed client's identity.

**Example**

```bash
curl -fsSL \
    -XDELETE ${ENDPOINT}/accounts/me/auth/123.oauth2.example.org \
    -H"Authorization: Bearer ${ACCESS_TOKEN}" \
    | jq '.'

{
  "id": "123.oauth2.example.org"
}
```