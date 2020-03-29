# AWS STS Token fetching and refreshing

As we use multiple accounts it can be sometimes bothersome to have to switch between them. 

We use the current credentials and a token code to get a session token; this token is then used to fetch credentials 
for a number of different roles via the assume role api.

To get started first add the primary MFA identifier you want to use:

```shell script
$ awsts mfa arn:aws:iam::123456789:mfa/user@example.com
```

Now try to fetch the initial session token:

```shell script
$ awsts login
Enter MFA Code: 123456
```

Now you can add roles that you should be able to fetch temporary credentials for:

```shell script
$ awsts role ls
$ awsts role add --name dev --arn arn:aws:iam::123456789:role/Pipelines
$ awsts role ls
dev: arn:aws:iam::123456789:role/Pipelines no credentials
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