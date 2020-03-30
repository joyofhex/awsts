extern crate structopt;

mod config;
mod error;

use structopt::StructOpt;
use rusoto_sts::{StsClient, Sts, GetSessionTokenRequest};
use std::time::Duration;
use rusoto_core::{HttpClient, Region};
use rusoto_core::credential::ChainProvider;
use std::io;
use std::io::Write;
use std::collections::HashMap;
use error::CliError;
use std::process::exit;

type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, StructOpt)]
enum RoleOptions {
    List {},
    Remove {
        name: String,
    },
    Add {
        name: String,
        arn: String,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Managed access to AWS Roles via STS")]
enum CliOptions {
    Config {
        #[structopt(long)]
        serial_number: Option<String>,
        #[structopt(long)]
        session_name: Option<String>,
    },
    Login {},
    Role {
        #[structopt(subcommand)]
        cmd: RoleOptions
    },
    Fetch {
        name: String,
    },
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => exit(0),
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }
}

async fn run() -> Result<()> {
    let program_name = "awsts";
    let mut config = config::CliConfig::load(&program_name)?;
    let args = CliOptions::from_args();

    match args {
        CliOptions::Config { serial_number, session_name } => {
            if let Some(serial_number) = serial_number {
                config.set_mfa(&serial_number)?;
            }

            if let Some(session_name) = session_name {
                config.set_session_name(&session_name)?;
            }
        },
        CliOptions::Login {} => aws_login(config).await?,
        CliOptions::Role { cmd } => match cmd {
            RoleOptions::List {} => {
                let roles = config.get_roles();
                print_roles(roles);
            },
            RoleOptions::Remove { name } => config.remove_role(&name)?,
            RoleOptions::Add { name, arn} => config.add_role(&name, &arn)?,

        },
        CliOptions::Fetch { name } => config.fetch_sts(&name).await?,
    }

    Ok(())
}

async fn aws_login(mut config: config::CliConfig) -> Result<()> {
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

fn print_roles(roles: HashMap<String, String>) {
    println!("{:10} {}", "Name", "ARN");
    for (name, arn) in roles {
        println!("{:10} {}", name, arn);
    }
}

