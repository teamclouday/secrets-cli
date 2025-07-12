use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::Select;

mod tools;

#[derive(Parser)]
#[command(
    name = "tc-secrets",
    version = "1.0",
    about = "A CLI tool for synchronizing .env secret files with AWS Secrets Manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Authenticate with AWS Secrets Manager")]
    Auth,
    #[command(about = "Display differences between local and remote secret files")]
    Diff {
        #[arg(help = "Path to the local secret file", short, long)]
        filepath: String,
        #[arg(
            help = "Optional password for decrypting the secret file",
            short,
            long,
            default_value = "secret"
        )]
        password: String,
    },
    #[command(about = "Increment the version of the local secret file")]
    Bump {
        #[arg(help = "Path to the local secret file", short, long)]
        filepath: String,
    },
    #[command(about = "Reset the local secret file by the remote secret")]
    Reset {
        #[arg(help = "Path to the local secret file", short, long)]
        filepath: String,
        #[arg(
            help = "Optional password for decrypting the secret file",
            short,
            long,
            default_value = "secret"
        )]
        password: String,
    },
    #[command(about = "Synchronize local secret file with the remote secret")]
    Sync {
        #[arg(help = "Path to the local secret file", short, long)]
        filepath: String,
        #[arg(
            help = "Optional password for decrypting the secret file",
            short,
            long,
            default_value = "secret"
        )]
        password: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), tools::CliError> {
    let mut aws_client = tools::AWS::new().await?;

    match cli.command {
        Commands::Auth => {
            // only reauthenticate if the client was created from cache
            if aws_client.from_cache {
                aws_client.reauthenticate().await?;
            }
        }
        Commands::Diff { filepath, password } => {
            let encryption = tools::Encryption::new(password);

            // check if the file exists
            let path = std::path::PathBuf::from(filepath.clone());
            if !path.exists() {
                return Err(tools::CliError::InvalidEnvFileError(format!(
                    "The file '{}' does not exist.",
                    filepath
                )));
            }

            // load the local secret file
            let env_file = tools::EnvFile::new_local(path)?;

            let secret_id = env_file.secret_id.ok_or_else(|| {
                tools::CliError::InvalidEnvFileError(
                    "The secret file does not contain a secret ID.".to_string(),
                )
            })?;
            let field_id = env_file.field_id.ok_or_else(|| {
                tools::CliError::InvalidEnvFileError(
                    "The secret file does not contain a field ID.".to_string(),
                )
            })?;

            // load the remote secret from AWS Secrets Manager
            let aws_secret =
                tools::AWSSecret::new(aws_client.load_secret(secret_id.clone()).await?)?;

            // decrypt the remote secret
            let decrypted_remote_secret =
                encryption.decrypt(aws_secret.load_field(field_id.clone())?)?;

            // display diff
            println!(
                "Comparing local file {} with remote secret {}",
                style(filepath).magenta(),
                style(format!("{}/{}", secret_id, field_id)).cyan()
            );
            tools::display_diff(decrypted_remote_secret, env_file.content);
        }
        Commands::Bump { filepath } => {
            let path = std::path::PathBuf::from(filepath.clone());

            if !path.exists() {
                return Err(tools::CliError::InvalidEnvFileError(format!(
                    "The file '{}' does not exist.",
                    filepath
                )));
            }

            // load the local secret file
            let mut env_file = tools::EnvFile::new_local(path)?;

            // increment the version
            let new_version = env_file.version.unwrap_or(0) + 1;
            env_file.version = Some(new_version);

            // write the updated file
            env_file.write()?;

            println!(
                "Bumped the version of the secret file to {}",
                style(new_version).cyan()
            );
        }
        Commands::Reset { filepath, password } => {
            let encryption = tools::Encryption::new(password);
            let path = std::path::PathBuf::from(filepath.clone());

            // load the local secret file
            let mut local_env_file = tools::EnvFile::new_local(path.clone())?;

            let aws_secret = load_env_info(&aws_client, &filepath, &mut local_env_file).await?;

            let secret_id = local_env_file.secret_id.ok_or_else(|| {
                tools::CliError::InvalidEnvFileError(
                    "The secret file does not contain a secret ID.".to_string(),
                )
            })?;
            let field_id = local_env_file.field_id.ok_or_else(|| {
                tools::CliError::InvalidEnvFileError(
                    "The secret file does not contain a field ID.".to_string(),
                )
            })?;

            // load the remote secret from AWS Secrets Manager
            let decrypted_remote_secret =
                encryption.decrypt(aws_secret.load_field(field_id.clone())?)?;

            let mut remote_env_file = tools::EnvFile::new_remote(decrypted_remote_secret)?;
            remote_env_file.secret_id = remote_env_file.secret_id.or(Some(secret_id.clone()));
            remote_env_file.field_id = remote_env_file.field_id.or(Some(field_id.clone()));
            remote_env_file.version = remote_env_file.version.or(Some(1));
            remote_env_file.filepath = Some(path.clone());

            if path.exists() {
                if dialoguer::Confirm::new()
                    .with_prompt(format!(
                        "The file {} already exists. Do you want to make a `.bak` backup?",
                        style(filepath.clone()).cyan()
                    ))
                    .default(true)
                    .interact()
                    .expect("Failed to confirm backup")
                {
                    // create a backup of the local file
                    let backup_path = path.with_extension("bak");
                    std::fs::copy(&path, &backup_path).map_err(|e| tools::CliError::IoError(e))?;
                } else {
                    println!(
                        "Overwriting file {} with remote secret",
                        style(filepath.clone()).cyan()
                    );
                }
            }

            remote_env_file.write()?;

            println!(
                "Local secret file has been reset by remote secret {}",
                style(format!("{}/{}", secret_id, field_id)).cyan()
            );
        }
        Commands::Sync { filepath, password } => {
            let encryption = tools::Encryption::new(password);
            let path = std::path::PathBuf::from(filepath.clone());

            // load the local secret file
            let mut local_env_file = tools::EnvFile::new_local(path.clone())?;
            let mut aws_secret = load_env_info(&aws_client, &filepath, &mut local_env_file).await?;

            let secret_id = local_env_file.secret_id.ok_or_else(|| {
                tools::CliError::InvalidEnvFileError(
                    "The secret file does not contain a secret ID.".to_string(),
                )
            })?;
            let field_id = local_env_file.field_id.ok_or_else(|| {
                tools::CliError::InvalidEnvFileError(
                    "The secret file does not contain a field ID.".to_string(),
                )
            })?;

            println!(
                "Synchronizing with remote secret {}",
                style(format!("{}/{}", secret_id, field_id)).cyan()
            );

            // load the remote secret from AWS Secrets Manager
            let decrypted_remote_secret =
                encryption.decrypt(aws_secret.load_field(field_id.clone())?)?;

            let mut remote_env_file = tools::EnvFile::new_remote(decrypted_remote_secret)?;
            remote_env_file.secret_id = remote_env_file.secret_id.or(Some(secret_id.clone()));
            remote_env_file.field_id = remote_env_file.field_id.or(Some(field_id.clone()));

            // compare the local and remote versions
            let local_version = local_env_file.version.clone().unwrap_or(0);
            let remote_version = remote_env_file.version.clone().unwrap_or(0);

            if local_version < remote_version {
                remote_env_file.filepath = Some(path.clone());
                remote_env_file.write()?;

                println!(
                    "The local secret file is outdated. Updated to version {}",
                    style(remote_version).cyan()
                );
            } else if local_version > remote_version {
                let encrypted_content = encryption.encrypt(local_env_file.content.clone())?;
                aws_secret.put_field(field_id.clone(), encrypted_content.clone())?;
                aws_client
                    .put_secret(secret_id, aws_secret.to_string()?)
                    .await?;

                println!(
                    "The remote secret has been updated with the local secret file version {}",
                    style(local_version).cyan()
                );
            } else if local_env_file.content != remote_env_file.content {
                println!(
                    "The local secret file has same version but different content. Please run `tc-secrets diff` to see the differences."
                );
            } else {
                println!("The local secret file is up to date!");
            }
        }
    }

    Ok(())
}

async fn load_env_info(
    aws_client: &tools::AWS,
    filepath: &str,
    env_file: &mut tools::EnvFile,
) -> Result<tools::AWSSecret, tools::CliError> {
    // get the secret ID from the local file
    if env_file.secret_id.is_none() {
        let remote_secrets = aws_client.list_secrets().await?;

        let selection = Select::new()
            .with_prompt(format!(
                "No local file found at {}. Please select a secret ID to download:",
                style(filepath).cyan()
            ))
            .items(&remote_secrets)
            .default(0)
            .interact()
            .expect("Failed to select a secret ID");

        env_file.secret_id = Some(remote_secrets[selection].clone());
    }

    let secret_id = env_file.secret_id.clone().unwrap();

    // create a new AWSSecret instance
    let aws_secret = tools::AWSSecret::new(aws_client.load_secret(secret_id.clone()).await?)?;

    // get the field ID from the local file
    if env_file.field_id.is_none() {
        let remote_fields = aws_secret.list_fields();

        let selection = Select::new()
            .with_prompt(format!(
                "Please select a field ID to load from secret {}:",
                style(secret_id.clone()).cyan()
            ))
            .items(&remote_fields)
            .default(0)
            .interact()
            .expect("Failed to select a field ID");

        env_file.field_id = Some(remote_fields[selection].clone());
    }

    Ok(aws_secret)
}
