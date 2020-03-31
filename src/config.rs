
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::fs::{File, create_dir_all};
use rusoto_sts::Credentials as RusotoCredentials;
use std::collections::HashMap;
use crate::error::CliError;

pub struct CliConfig {
    path: PathBuf,
    options: Config,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub expiration: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    mfa_serial_number: String,
    session_name: String,
    region: Option<String>,
    roles: HashMap<String, String>,
    session_token: Option<Credentials>,
}

impl CliConfig {
    pub fn load(program_name: &str) -> super::Result<CliConfig> {
        let path = Self::construct_path(program_name)?;

        if path.is_file() {
            let contents = std::fs::read_to_string(&path)?;
            let options: Config = serde_json::from_str(&contents)?;
            Ok(CliConfig { path, options })
        } else {
            let options = Config {
                session_name: "default".to_string(),
                region: Some("us-east-1".to_string()),
                ..Default::default()
            };
            Ok(CliConfig { path, options })
        }
    }

    fn save(&self) -> super::Result<()> {
        let directory = self.path.parent().expect("Parent path could not be extracted.");

        create_dir_all(directory)?;
        let file = File::create(&self.path)?;
        serde_json::to_writer_pretty(file, &self.options)?;

        Ok(())
    }

    fn construct_path(program_name: &str) -> super::Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or(CliError::ConfigDirectoryNotAvailable())?;

        path.push(program_name);
        path.push("config");
        Ok(path)
    }

    pub fn set_mfa(&mut self, serial_number: &str) -> super::Result<()> {
        self.options.mfa_serial_number = serial_number.to_string();
        self.save()?;

        Ok(())
    }

    pub fn get_mfa(&self) -> String {
        self.options.mfa_serial_number.to_string()
    }

    pub fn set_session_name(&mut self, session_name: &str) -> super::Result<()> {
        self.options.session_name = session_name.to_string();
        self.save()?;

        Ok(())
    }

    pub fn get_session_name(&self) -> String {
        self.options.session_name.to_string()
    }

    pub fn set_session_token(&mut self, credentials: RusotoCredentials) -> super::Result<()> {
        let credentials= Credentials::from(credentials);
        self.options.session_token = Some(credentials);
        self.save()?;

        Ok(())
    }

    pub fn get_session_token(&self) -> Option<Credentials> {
        self.options.session_token.clone()
    }

    pub fn add_role(&mut self, name: &str, arn: &str) -> super::Result<()> {
        self.options.roles.insert(name.to_string(), arn.to_string());
        self.save()?;

        Ok(())
    }

    pub fn get_roles(&self) -> HashMap<String, String> {
        self.options.roles.to_owned()
    }

    pub fn remove_role(&mut self, name: &str) -> super::Result<()> {
        self.options.roles.remove(name);
        self.save()?;

        Ok(())
    }

    pub fn get_role_arn(&self, name: &str) -> Option<&String> {
        self.options.roles.get(name)
    }

    pub fn get_region(&self) -> String {
        match &self.options.region {
            Some(region) => region.clone(),
            None => "us-east-1".to_string(),
        }
    }

    pub fn set_region(&mut self, region: &str) -> super::Result<()> {
        self.options.region = Some(region.to_owned());
        self.save()?;
        Ok(())
    }
}

impl From<RusotoCredentials> for Credentials {
    fn from(credentials: RusotoCredentials) -> Self {
        Credentials {
            access_key_id: credentials.access_key_id,
            secret_access_key: credentials.secret_access_key,
            session_token: credentials.session_token,
            expiration: credentials.expiration,
        }
    }
}

impl Clone for Credentials {
    fn clone(&self) -> Credentials {
        Credentials {
            access_key_id: self.access_key_id.to_owned(),
            secret_access_key: self.secret_access_key.to_owned(),
            session_token: self.session_token.to_owned(),
            expiration: self.expiration.to_owned(),
        }
    }
}