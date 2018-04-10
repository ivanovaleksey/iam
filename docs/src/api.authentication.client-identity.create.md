# Create

Add another client's identity to the account.

*NOTE: the operation isn't allowed for disabled accounts*

**URI**

```
POST /auth/${KEY}/link
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
KEY       | string | _required_ | Account identifier or `me`

**Payload**

Name          | Type   | Default    | Description
------------- | ------ | ---------- | ------------------
grant\_type   | string | _required_ | Always `client_credentials`
client\_token | string | _required_ | Client credentials

**Response**

Name           | Type   | Default    | Description
-------------- | ------ | ---------- | ------------------
id             | string | _required_ | Client's identity identifier

**Example**

```bash
## www-form payload
curl -fsSL \
    -XPOST ${ENDPOINT}/auth/oauth2.example.org/link \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H 'Content-Type: application/x-www-form-urlencoded' \
    -d "grant_type=client_credentials&client_token=${CLIENT_TOKEN}" \
    | jq '.'
 
## JSON payload
curl -fsSL \
    -XPOST ${ENDPOINT}/auth/oauth2.example.org/link \
    -H "Authorization: Bearer ${ACCESS_TOKEN}" \
    -H 'Content-Type: application/json' \
    -d '{"grant_type":"client_credentials","client_token":"${CLIENT_TOKEN}"}' \
    | jq '.'
 
{
  "id": "123.oauth2.example.org"
}
```