use std::collections::HashMap;

use anyhow::Result;
use aws_config::SdkConfig;
use aws_config::meta::region::RegionProviderChain;
use aws_config::profile::ProfileFileCredentialsProvider;
use aws_sdk_secretsmanager::Client;
use aws_sdk_sts::Client as StsClient;
use console::style;
use serde_json;

use super::error::CliError;

pub struct AWS {
    client: Client,
    pub from_cache: bool,
}

const PROFILE_NAME: &str = "tc-secrets-cli-profile";

impl AWS {
    pub async fn new() -> Result<Self, CliError> {
        let provider = ProfileFileCredentialsProvider::builder()
            .profile_name(PROFILE_NAME)
            .build();

        let config: aws_config::SdkConfig =
            aws_config::defaults(aws_config::BehaviorVersion::latest())
                .credentials_provider(provider)
                .region(RegionProviderChain::default_provider().or_else("us-east-1"))
                .load()
                .await;

        let (client, from_cache) = match Self::report_account_info(&config).await {
            Ok(()) => (Client::new(&config), true),
            Err(_e) => {
                println!("AWS credentials expired or invalid. Authenticating...");
                (Self::authenticate().await?, false)
            }
        };
        Ok(AWS { client, from_cache })
    }

    pub async fn reauthenticate(&mut self) -> Result<(), CliError> {
        let client = Self::authenticate().await?;
        self.client = client;

        Ok(())
    }

    pub async fn load_secret(&self, secret_id: String) -> Result<String, CliError> {
        // send the request to get the secret value
        let resp = self
            .client
            .get_secret_value()
            .secret_id(secret_id.clone())
            .send()
            .await
            .map_err(|e| CliError::AwsSecretsManagerError(e.to_string()))?;

        // get the secret value
        let secret_value = resp.secret_string().ok_or_else(|| {
            CliError::AwsSecretsManagerError(format!(
                "cannot load secret value content for {}",
                secret_id
            ))
        })?;

        Ok(secret_value.to_string())
    }

    pub async fn put_secret(
        &self,
        secret_id: String,
        secret_value: String,
    ) -> Result<(), CliError> {
        // send the request to put the secret value
        self.client
            .put_secret_value()
            .secret_id(secret_id)
            .secret_string(secret_value)
            .send()
            .await
            .map_err(|e| CliError::AwsSecretsManagerError(e.to_string()))?;

        Ok(())
    }

    pub async fn list_secrets(&self) -> Result<Vec<String>, CliError> {
        // send the request to list secrets
        let resp = self
            .client
            .list_secrets()
            .send()
            .await
            .map_err(|e| CliError::AwsSecretsManagerError(e.to_string()))?;

        // extract the secret names from the response
        let secrets = resp
            .secret_list()
            .iter()
            .filter_map(|s| s.name().map(String::from))
            .collect();

        Ok(secrets)
    }

    async fn authenticate() -> Result<Client, CliError> {
        // configure the profile
        let configure_status = std::process::Command::new("aws")
            .args(&["configure", "--profile", PROFILE_NAME])
            .status()
            .map_err(|e| CliError::AwsAuthError(e.to_string()))?;

        if !configure_status.success() {
            return Err(CliError::AwsAuthError(
                "Failed to configure AWS profile.".to_string(),
            ));
        }

        // create the credentials provider
        let provider = ProfileFileCredentialsProvider::builder()
            .profile_name(PROFILE_NAME)
            .build();

        let config: aws_config::SdkConfig =
            aws_config::defaults(aws_config::BehaviorVersion::latest())
                .credentials_provider(provider)
                .region(RegionProviderChain::default_provider().or_else("us-east-1"))
                .load()
                .await;

        Self::report_account_info(&config).await?;

        Ok(Client::new(&config))
    }

    async fn report_account_info(config: &SdkConfig) -> Result<(), CliError> {
        let client = StsClient::new(config);
        let resp = client
            .get_caller_identity()
            .send()
            .await
            .map_err(|e| CliError::AwsAuthError(e.to_string()))?;

        println!(
            "AWS Account ID: {}\nAWS User ID: {}\n",
            style(resp.account().unwrap_or("Unknown")).cyan(),
            style(resp.user_id().unwrap_or("Unknown")).cyan()
        );
        Ok(())
    }
}

pub struct AWSSecret {
    pub data: HashMap<String, String>,
}

impl AWSSecret {
    pub fn new(secret: String) -> Result<Self, CliError> {
        let data: HashMap<String, String> = serde_json::from_str(&secret)
            .map_err(|e| CliError::AwsSecretsFormatError(e.to_string()))?;

        Ok(AWSSecret { data })
    }

    pub fn list_fields(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    pub fn load_field(&self, field_id: String) -> Result<String, CliError> {
        self.data.get(field_id.as_str()).cloned().ok_or_else(|| {
            CliError::AwsSecretsFormatError(format!("Field '{}' not found", field_id))
        })
    }

    pub fn put_field(&mut self, field_id: String, value: String) -> Result<(), CliError> {
        self.data.insert(field_id, value);
        Ok(())
    }

    pub fn to_string(&self) -> Result<String, CliError> {
        serde_json::to_string(&self.data)
            .map_err(|e| CliError::AwsSecretsFormatError(e.to_string()))
    }
}
