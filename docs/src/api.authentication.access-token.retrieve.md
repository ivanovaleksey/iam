# Retrieve

Issues access and refresh tokens of account. For **OAuth2 Client Credentials Grant** authentication flow client's credentials are used to identify the subject of authentication. If an account hasn't exit yet it will be created.

*NOTE: the operation isn't allowed for disabled accounts*

**URI**

```
POST /auth/${AUTH_KEY}/token
```

**URI parameters**

Name      | Type   | Default    | Description
--------- | ------ | ---------- | ------------------
AUTH\_KEY | string | _required_ | Authentication key (follows `${LABEL}.${PROVIDER}` convention)

**Payload**

Name          | Type   | Default    | Description
------------- | ------ | ---------- | ------------------
grant\_type   | string | _required_ | Always `client_credentials`
client\_token | string | _required_ | Client credentials
expires\_in   | int    |        300 | Desired expiration time

**Response**

Name           | Type   | Default    | Description
-------------- | ------ | ---------- | ------------------
access\_token  | string | _required_ | Used for account identification
refresh\_token | string | _required_ | Used to refresh the access token, never expires
expires\_in    | int    | _required_ | Expiration time of access token
token\_type    | string | _required_ | Always `Bearer`

**Example**

```bash
## www-form payload
curl -fsSL \
    -XPOST ${ENDPOINT}/auth/oauth2.example.org/token \
    -H 'Content-Type: application/x-www-form-urlencoded' \
    -d "grant_type=client_credentials&client_token=${CLIENT_TOKEN}" \
    | jq '.'
 
## JSON payload
curl -fsSL \
    -XPOST ${ENDPOINT}/auth/oauth2.example.org/token \
    -H 'Content-Type: application/json' \
    -d "{\"grant_type\":\"client_credentials\",\"client_token\":\"${CLIENT_TOKEN}\"}" \
    | jq '.'
 
{
  "access_token": "eyJhbGci...",
  "refresh_token": "eyJhbGci...",
  "expires_in": 86400,
  "token_type": "Bearer"
}
```
