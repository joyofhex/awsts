use crate::config;
use std::io;
use std::time::Duration;
use rusoto_sts::{StsClient, GetSessionTokenRequest, Sts, AssumeRoleRequest};
use rusoto_core::{HttpClient, Region};
use rusoto_core::credential::{AwsCredentials, ChainProvider, StaticProvider};
use crate::error::CliError;
use std::io::Write;
use chrono::{DateTime, Utc};
use crate::config::Credentials;


pub struct AwsSts {}

impl AwsSts {
    pub async fn login(mut config: config::CliConfig) -> super::Result<()> {
        print!("Enter token code for MFA ({}): ", config.get_mfa());
        io::stdout().flush().ok().expect("Could not flush stdout");

        let mut token_code = String::new();
        io::stdin().read_line(&mut token_code)?;
        let code = token_code.trim_end();

        let mut provider = ChainProvider::new();
        provider.set_timeout(Duration::from_secs(2));

        let sts = StsClient::new_with(
            HttpClient::new().expect("failed"),
            provider,
            Region::EuWest1
        );

        let get_token_request = GetSessionTokenRequest {
            token_code: Some(code.to_string()),
            serial_number: Some(config.get_mfa()),
            ..Default::default()
        };

        let credentials = sts.get_session_token(get_token_request)
            .await?
            .credentials
            .ok_or(CliError::NoCredentialsInResponse())?;

        config.set_session_token(credentials)?;

        Ok(())
    }

    pub async fn fetch_sts(config: config::CliConfig, name: &str) -> super::Result<Credentials> {
        let token = config.get_session_token()
            .ok_or(CliError::NoSessionToken())?;

        let expiry = DateTime::parse_from_rfc3339(&token.expiration)?
            .with_timezone(&Utc);

        let credentials = AwsCredentials::new(
            &token.access_key_id,
            &token.secret_access_key,
            Some(token.session_token.to_owned()),
            Some(expiry),
        );

        let provider = StaticProvider::from(credentials);
        let sts = StsClient::new_with(
            HttpClient::new().expect("failed"),
            provider,
            Region::EuWest1,
        );

        let arn = config.get_role_arn(name)
            .ok_or(CliError::RoleNotFound(name.to_string()))?;

        let assume_role_result = sts.assume_role(
            AssumeRoleRequest {
                role_arn: arn.to_owned(),
                role_session_name: config.get_session_name(),
                ..Default::default()
            }
        ).await?;

        let sts_credentials = assume_role_result.credentials
            .ok_or(CliError::NoCredentialsInResponse())?;

        Ok(Credentials::from(sts_credentials))
    }
}
