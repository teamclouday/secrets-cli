use anyhow::Result;
use magic_crypt::{MagicCrypt256, MagicCryptTrait, new_magic_crypt};

use super::error::CliError;

pub struct Encryption {
    mcrypt: MagicCrypt256,
}

impl Encryption {
    pub fn new(password: String) -> Self {
        let mcrypt = new_magic_crypt!(password, 256);
        Encryption { mcrypt }
    }

    pub fn encrypt(&self, data: String) -> Result<String, CliError> {
        if data.is_empty() {
            return Ok("".to_string());
        }

        // encrypt content
        let encrypted_content = self.mcrypt.encrypt_str_to_base64(&data);

        if encrypted_content.is_empty() {
            Err(CliError::EncryptionError(
                "Encrypted content is empty".to_string(),
            ))
        } else {
            Ok(encrypted_content)
        }
    }

    pub fn decrypt(&self, data: String) -> Result<String, CliError> {
        if data.is_empty() {
            return Ok("".to_string());
        }

        // decrypt content
        let content = self
            .mcrypt
            .decrypt_base64_to_string(data)
            .map_err(|e| CliError::DecryptionError(e.to_string()))?;

        if content.is_empty() {
            Err(CliError::DecryptionError(
                "Decrypted content is empty".to_string(),
            ))
        } else {
            Ok(content)
        }
    }
}
