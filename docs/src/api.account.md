# Account

### Description

Account is meant to be a "virtual" entity used to link user identities of different providers.   
That is why Account API is very concise.  

An account gets automatically created on creating an identity (via Identity API or Authentication API).  
On deleting the last user's identity an account is marked as _deleted_ and no longer available to use. 

## Methods
- [Read](api.account.read.html)
