extern crate structopt;

mod config;
mod error;
mod aws_sts;

use structopt::StructOpt;
use std::collections::HashMap;
use error::CliError;
use std::process::exit;
use aws_sts::AwsSts;

type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, StructOpt)]
enum RoleOptions {
    List {},
    Remove {
        name: String,
    },
    Add {
        #[structopt(help = "Short name to identify role")]
        name: String,
        #[structopt(help = "ARN of role to assume")]
        arn: String,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(author, about = "Managed access to AWS Roles via STS")]
enum CliOptions {
    Config {
        #[structopt(long, help = "Set MFA serial number")]
        serial_number: Option<String>,
        #[structopt(long, help = "Set session name")]
        session_name: Option<String>,
        #[structopt(long, help = "Set AWS Region name")]
        region: Option<String>,
    },
    Login {
        #[structopt(long, help = "Set credential profile to use")]
        profile: Option<String>
    },
    Role {
        #[structopt(subcommand)]
        cmd: RoleOptions
    },
    Fetch {
        #[structopt(help = "Role to assume")]
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
        CliOptions::Config { serial_number, session_name, region } => {
            if serial_number == None && session_name == None && region == None {
                print_config(&config);
            }

            if let Some(serial_number) = serial_number {
                config.set_mfa(&serial_number)?;
            }

            if let Some(session_name) = session_name {
                config.set_session_name(&session_name)?;
            }

            if let Some(region) = region {
                config.set_region(&region)?;
            }

        },
        CliOptions::Login { profile } => {
            AwsSts::login(config, profile).await?
        },
        CliOptions::Role { cmd } => match cmd {
            RoleOptions::List {} => {
                let roles = config.get_roles();
                print_roles(roles);
            },
            RoleOptions::Remove { name } => config.remove_role(&name)?,
            RoleOptions::Add { name, arn} => config.add_role(&name, &arn)?,

        },
        CliOptions::Fetch { name } => {
            let sts_credentials = AwsSts::fetch_sts(config, &name).await?;
            println!("export AWS_ACCESS_KEY_ID={}", sts_credentials.access_key_id);
            println!("export AWS_SECRET_ACCESS_KEY={}", sts_credentials.secret_access_key);
            println!("export AWS_SESSION_TOKEN={}", sts_credentials.session_token);
            println!("export AWS_CREDENTIAL_EXPIRATION={}", sts_credentials.expiration);
        },
    }

    Ok(())
}

fn print_config(config: &config::CliConfig) {
    println!("MFA: {}", config.get_mfa());
    println!("Session Name: {}", config.get_session_name());
    println!("Region: {}", config.get_region());
    println!("\nRoles:");
    print_roles(config.get_roles());

}

fn print_roles(roles: HashMap<String, String>) {
    println!("{:10} ARN", "Name");
    for (name, arn) in roles {
        println!("{:10} {}", name, arn);
    }
}

