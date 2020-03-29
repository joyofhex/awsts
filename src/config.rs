
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::fs::{File, create_dir_all};
use rusoto_sts::{Credentials, StsClient, Sts, AssumeRoleRequest};
use rusoto_core::credential::{AwsCredentials, StaticProvider};
use rusoto_core::{Region, HttpClient};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub struct CliConfig {
    path: PathBuf,
    options: Config,
}

#[derive(Debug, Serialize, Deserialize)]
struct CredentialsDef {
    access_key_id: String,
    secret_access_key: String,
    session_token: String,
    expiration: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    mfa_serial_number: String,
    session_name: String,
    roles: HashMap<String, String>,
    session_token: Option<CredentialsDef>,
}

impl CliConfig {
    pub fn load(program_name: &str) -> CliConfig {
        let path = Self::construct_path(program_name);
        let contents = std::fs::read_to_string(&path).unwrap();
        let options= serde_json::from_str(&contents).unwrap();
        CliConfig { path, options }
    }

    fn save(&self) {
        let directory = self.path.parent().unwrap();
        create_dir_all(directory).unwrap();
        let file = File::create(&self.path).unwrap();
        serde_json::to_writer_pretty(file, &self.options).unwrap();
    }

    fn construct_path(program_name: &str) -> PathBuf {
        let mut path = dirs::config_dir().unwrap();
        path.push(program_name);
        path.push("config");
        path
    }

    pub fn set_mfa(&mut self, serial_number: &str) {
        self.options.mfa_serial_number = serial_number.to_string();
        self.save();
    }

    pub fn get_mfa(&self) -> String {
        self.options.mfa_serial_number.to_string()
    }

    pub fn set_session_name(&mut self, session_name: &str) {
        self.options.session_name = session_name.to_string();
        self.save();
    }

    pub fn set_session_token(&mut self, credentials: Credentials) {
        let credentials= CredentialsDef::from(credentials);
        self.options.session_token = Some(credentials);
        println!("opts: {:?}", self.options);
        self.save();
    }

    pub fn add_role(&mut self, name: &str, arn: &str) {
        self.options.roles.insert(name.to_string(), arn.to_string());
        self.save();
    }

    pub fn get_roles(&self) -> HashMap<String, String> {
        self.options.roles.to_owned()
    }

    pub fn remove_role(&mut self, name: &str) {
        self.options.roles.remove(name);
        self.save();
    }

    pub async fn fetch_sts(&mut self, name: &str) {
        let token = self.options.session_token.as_ref().unwrap();
        let expiry= DateTime::parse_from_rfc3339(&token.expiration).unwrap().with_timezone(&Utc);
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

        let assume_role_result = sts.assume_role(
            AssumeRoleRequest {
                role_arn: self.options.roles.get(name).unwrap().to_owned(),
                role_session_name: self.options.session_name.to_owned(),
                ..Default::default()
            }
        ).await.unwrap();
        let sts_credentials = assume_role_result.credentials.unwrap();
        println!("export AWS_ACCESS_KEY_ID={}", sts_credentials.access_key_id);
        println!("export AWS_SECRET_ACCESS_KEY={}", sts_credentials.secret_access_key);
        println!("export AWS_SESSION_TOKEN={}", sts_credentials.session_token);
        println!("export AWS_CREDENTIAL_EXPIRATION={}", sts_credentials.expiration);

    }
}

impl From<Credentials> for CredentialsDef {
    fn from(credentials: Credentials) -> Self {
        CredentialsDef {
            access_key_id: credentials.access_key_id,
            secret_access_key: credentials.secret_access_key,
            session_token: credentials.session_token,
            expiration: credentials.expiration,
        }
    }
}