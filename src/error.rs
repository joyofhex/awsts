use rusoto_sts::{AssumeRoleError, GetSessionTokenError};
use std::error::Error;
use std::{fmt, io};
// use rusoto_core::credential::CredentialsError;

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Serde(serde_json::Error),
    RoleNotFound(String),
    NoSessionToken(),
    DateTimeParseFailure(chrono::format::ParseError),
    RusotoError(String),
    NoCredentialsInResponse(),
    ConfigDirectoryNotAvailable(),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Io(ref err) => err.fmt(f),
            CliError::Serde(ref err) => err.fmt(f),
            CliError::RoleNotFound(ref role) => write!(f, "Role {} not found", role),
            CliError::NoSessionToken() => {
                write!(f, "No session token, use awsts login to fetch one")
            }
            CliError::DateTimeParseFailure(ref err) => err.fmt(f),
            CliError::RusotoError(ref err) => err.fmt(f),
            CliError::NoCredentialsInResponse() => write!(f, "No credentials provided in response"),
            CliError::ConfigDirectoryNotAvailable() => {
                write!(f, "Platform has no available config directory")
            }
        }
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            CliError::Io(ref err) => Some(err),
            CliError::Serde(ref err) => Some(err),
            CliError::RoleNotFound(ref _role) => None,
            CliError::NoSessionToken() => None,
            CliError::DateTimeParseFailure(ref err) => Some(err),
            CliError::RusotoError(ref _err) => None,
            CliError::NoCredentialsInResponse() => None,
            CliError::ConfigDirectoryNotAvailable() => None,
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> CliError {
        CliError::Serde(err)
    }
}

impl From<chrono::format::ParseError> for CliError {
    fn from(err: chrono::format::ParseError) -> CliError {
        CliError::DateTimeParseFailure(err)
    }
}

impl From<rusoto_core::RusotoError<AssumeRoleError>> for CliError {
    fn from(err: rusoto_core::RusotoError<AssumeRoleError>) -> CliError {
        CliError::RusotoError(err.to_string())
    }
}

impl From<rusoto_core::RusotoError<GetSessionTokenError>> for CliError {
    fn from(err: rusoto_core::RusotoError<GetSessionTokenError>) -> CliError {
        CliError::RusotoError(err.to_string())
    }
}

impl From<rusoto_core::credential::CredentialsError> for CliError {
    fn from(err: rusoto_core::credential::CredentialsError) -> CliError {
        CliError::RusotoError(err.to_string())
    }
}
