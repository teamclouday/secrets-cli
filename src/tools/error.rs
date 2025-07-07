use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to encrypt: {0}")]
    EncryptionError(String),
    #[error("Failed to decrypt: {0}")]
    DecryptionError(String),
    #[error("AWS Authentication Error: {0}")]
    AwsAuthError(String),
    #[error("AWS Secrets Manager Error: {0}")]
    AwsSecretsManagerError(String),
    #[error("AWS Secrets JSON Format Error: {0}")]
    AwsSecretsFormatError(String),
    #[error("Failed to parse the secrets file: {0}")]
    InvalidEnvFileError(String),
}
