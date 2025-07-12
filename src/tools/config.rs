use anyhow::Result;
use std::path::PathBuf;

use super::error::CliError;

pub struct EnvFile {
    pub filepath: Option<PathBuf>,
    pub content: String,
    pub version: Option<u32>,
    pub secret_id: Option<String>,
    pub field_id: Option<String>,
}

const SECRETS_VERSION_HEADER: &str = "#do-not-edit--secrets-version";
const SECRETS_ID_HEADER: &str = "#do-not-edit--secrets-id";
const SECRETS_FIELD_ID_HEADER: &str = "#do-not-edit--secrets-field-id";

impl EnvFile {
    pub fn new_local(filepath: PathBuf) -> Result<Self, CliError> {
        let mut env_file = if filepath.exists() {
            let content = std::fs::read_to_string(&filepath).map_err(|e| CliError::IoError(e))?;

            EnvFile {
                filepath: Some(filepath),
                content,
                version: None,
                secret_id: None,
                field_id: None,
            }
        } else {
            EnvFile {
                filepath: Some(filepath),
                content: String::new(),
                version: None,
                secret_id: None,
                field_id: None,
            }
        };

        env_file.parse()?;
        Ok(env_file)
    }

    pub fn new_remote(content: String) -> Result<Self, CliError> {
        let mut env_file = EnvFile {
            filepath: None,
            content,
            version: None,
            secret_id: None,
            field_id: None,
        };

        env_file.parse()?;
        Ok(env_file)
    }

    fn parse(&mut self) -> Result<(), CliError> {
        // parse the header to extract version, secret_id and field_id
        for line in self.content.lines() {
            if line.starts_with(SECRETS_VERSION_HEADER) {
                if let Some(version) = line.split(' ').nth(1) {
                    self.version = Some(version.parse::<u32>().map_err(|_| {
                        CliError::InvalidEnvFileError("Invalid version in header".to_string())
                    })?);
                } else {
                    return Err(CliError::InvalidEnvFileError(
                        "Invalid version in header".to_string(),
                    ));
                }
            } else if line.starts_with(SECRETS_ID_HEADER) {
                if let Some(secret_id) = line.split(' ').nth(1) {
                    self.secret_id = Some(secret_id.to_string());
                } else {
                    return Err(CliError::InvalidEnvFileError(
                        "Invalid secret_id in header".to_string(),
                    ));
                }
            } else if line.starts_with(SECRETS_FIELD_ID_HEADER) {
                if let Some(field_id) = line.split(' ').nth(1) {
                    self.field_id = Some(field_id.to_string());
                } else {
                    return Err(CliError::InvalidEnvFileError(
                        "Invalid field_id in header".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn write(&mut self) -> Result<(), CliError> {
        let mut lines = self
            .content
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<String>>();

        // write/update the field_id
        if let Some(field_id) = &self.field_id {
            // find the line with #secrets-field-id
            let line_index = lines
                .iter()
                .position(|line| line.starts_with(SECRETS_FIELD_ID_HEADER));

            if let Some(index) = line_index {
                lines[index] = format!("{} {}", SECRETS_FIELD_ID_HEADER, field_id);
            } else {
                lines.insert(0, format!("{} {}", SECRETS_FIELD_ID_HEADER, field_id));
            }
        }

        // write/update the secret_id
        if let Some(secret_id) = &self.secret_id {
            // find the line with #secrets-id
            let line_index = lines
                .iter()
                .position(|line| line.starts_with(SECRETS_ID_HEADER));

            if let Some(index) = line_index {
                lines[index] = format!("{} {}", SECRETS_ID_HEADER, secret_id);
            } else {
                lines.insert(0, format!("{} {}", SECRETS_ID_HEADER, secret_id));
            }
        } else {
            return Err(CliError::InvalidEnvFileError(
                "Secret ID is not set".to_string(),
            ));
        }

        // write/update the version
        if let Some(version) = &self.version {
            // find the line with #secrets-version
            let line_index = lines
                .iter()
                .position(|line| line.starts_with(SECRETS_VERSION_HEADER));

            if let Some(index) = line_index {
                lines[index] = format!("{} {}", SECRETS_VERSION_HEADER, version);
            } else {
                lines.insert(0, format!("{} {}", SECRETS_VERSION_HEADER, version));
            }
        } else {
            return Err(CliError::InvalidEnvFileError(
                "Version is not set".to_string(),
            ));
        }

        self.content = lines.join("\n");

        if let Some(path) = self.filepath.clone() {
            std::fs::write(&path, &self.content).map_err(|e| CliError::IoError(e))?;
        }

        Ok(())
    }
}
