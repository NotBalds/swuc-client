use crate::changelog::ChangelogGenerator;
use anyhow::{Context, Result};
use chrono::Utc;
use directories::ProjectDirs;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_package_list(path: &str) -> Result<HashSet<String>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read package list from {}", path))?;

    Ok(content
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

pub fn get_cache_dir() -> Result<PathBuf> {
    let proj_dirs =
        ProjectDirs::from("com", "swuc", "SWUC").context("Failed to get project directories")?;

    let cache_dir = proj_dirs.cache_dir();
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir)
            .with_context(|| format!("Failed to create cache directory: {:?}", cache_dir))?;
    }

    Ok(cache_dir.to_path_buf())
}

pub fn save_cached_response(cache_dir: &Path, response: &str) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("response_{}.json", timestamp);
    let path = cache_dir.join(filename);

    fs::write(&path, response)
        .with_context(|| format!("Failed to write cached response to {:?}", path))?;

    Ok(())
}

pub fn process_server_response(response: &str, pathout: Option<&str>) -> Result<()> {
    let mut generator = ChangelogGenerator::new();
    generator.parse_response(response)?;

    // Directly save the output to pathout without creating an intermediate string
    if let Some(pathout) = pathout {
        generator.save_report_to(pathout)?;
    }
    Ok(())
}
