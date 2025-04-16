use anyhow::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::Local;
use std::collections::HashSet;
use std::time::Duration;
use tungstenite::{connect, Message};

use crate::config::UserConfig;
use crate::crypto;
use crate::storage;

pub fn check_updates_with_interval(
    user_cfg: &UserConfig,
    packages: &HashSet<String>,
    pathout: &str,
    interval: Option<u64>,
) -> Result<()> {
    loop {
        match check_updates(user_cfg, packages, pathout) {
            Ok(_) => println!("Update check completed at {}", Local::now()),
            Err(e) => eprintln!("Error during update check: {}", e),
        }

        if let Some(hours) = interval {
            std::thread::sleep(Duration::from_secs(hours * 3600));
        } else {
            break;
        }
    }

    Ok(())
}

pub fn check_updates(
    user_cfg: &UserConfig,
    packages: &HashSet<String>,
    pathout: &str,
) -> Result<()> {
    let response = send_update_request(user_cfg, packages)?;
    let cache_dir = storage::get_cache_dir()?;
    storage::save_cached_response(&cache_dir, &response)?;
    storage::process_server_response(&response, pathout)
}

fn send_update_request(user_cfg: &UserConfig, packages: &HashSet<String>) -> Result<String> {
    let (mut socket, _) = connect(&user_cfg.url)?;
    let encrypted_data = crypto::encrypt_request(packages, &user_cfg.public)?;

    let request = serde_json::json!({
        "uuid": user_cfg.uuid,
        "raw": BASE64_STANDARD.encode(&encrypted_data)
    });

    socket.send(Message::Text(
        BASE64_STANDARD.encode(request.to_string()).into(),
    ))?;

    match socket.read()? {
        Message::Text(resp) => crypto::decrypt_response(&resp, &user_cfg.secret),
        _ => anyhow::bail!("Unexpected response format"),
    }
}
