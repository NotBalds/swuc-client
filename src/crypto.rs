use anyhow::{Context, Result};
use base64::{prelude::BASE64_STANDARD, Engine};
use ecies::{decrypt, encrypt};
use std::collections::HashSet;

pub fn encrypt_request(packages: &HashSet<String>, public_key: &str) -> Result<Vec<u8>> {
    // Base64 encode each package name and join with pipes
    let plaintext = packages
        .iter()
        .map(|p| BASE64_STANDARD.encode(p))
        .collect::<Vec<_>>()
        .join("|");

    let pub_key = BASE64_STANDARD.decode(public_key)?;
    encrypt(&pub_key, plaintext.as_bytes()).map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))
}

pub fn decrypt_response(response: &str, secret_key: &str) -> Result<String> {
    let encrypted = BASE64_STANDARD.decode(response)?;
    let sec_key = BASE64_STANDARD.decode(secret_key)?;

    let decrypted =
        decrypt(&sec_key, &encrypted).map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    let decrypted_str =
        String::from_utf8(decrypted).context("Decrypted data is not valid UTF-8")?;

    // Parse server response JSON
    let response_json: serde_json::Value =
        serde_json::from_str(&decrypted_str).context("Failed to parse server response")?;

    response_json
        .get("software")
        .map(serde_json::to_string_pretty)
        .transpose()
        .context("Missing 'software' field in response")?
        .ok_or_else(|| anyhow::anyhow!("Software field is null"))
}
