# AWS STS Token fetching and refreshing

As we use multiple accounts it can be sometimes bothersome to have to switch between them. 

We use the current credentials and a token code to get a session token; this token is then used to fetch credentials 
for a number of different roles via the assume role api.

To get started first add the primary MFA identifier and session name you want to use:

```shell script
$ awsts config --serial-number arn:aws:iam::123456789:mfa/user@example.com --session-name user@example.com
```

Now try to fetch the initial session token:

```shell script
$ awsts login
Enter token code for MFA (arn:aws:iam::123456789:mfa/user@example.com): 123456
$
```

Now you can add roles that you should be able to fetch temporary credentials for:

```shell script
$ awsts role list
$ awsts role add --name dev --arn arn:aws:iam::123456789:role/Pipelines
$ awsts role list
Name       ARN
dev        arn:aws:iam::1234567890:role/Assumed-Role-Account
```

Finally you can fetch credentials for a specific role:

```shell script
$ awsts fetch dev
export AWS_ACCESS_KEY_ID=ASIA11111111111111
export AWS_SECRET_ACCESS_KEY=redacted
export AWS_SESSION_TOKEN=redacted
export AWS_CREDENTIAL_EXPIRATION=2020-03-27T16:45:20Z 
``` 

You'll probably want to source this.

```
$ . <(awsts fetch dev)
```
