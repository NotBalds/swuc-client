use anyhow::{Context, Result};
use chrono::Local;
use dirs::desktop_dir;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct PackageInfo {
    error: Option<serde_json::Value>,
    name: String,
    version: String,
    sources: Vec<String>,
    metadata: serde_json::Value,
}

pub struct ChangelogGenerator {
    short_changelog: String,
    long_changelog: String,
    packages: Vec<PackageInfo>,
}

impl ChangelogGenerator {
    pub fn new() -> Self {
        Self {
            short_changelog: String::new(),
            long_changelog: String::new(),
            packages: Vec::new(),
        }
    }

    pub fn parse_response(&mut self, response: &str) -> Result<()> {
        self.packages = serde_json::from_str(response)
            .context("Failed to parse package information from response")?;

        // Get desktop directory for the current versions file
        let desktop =
            desktop_dir().ok_or_else(|| anyhow::anyhow!("Desktop directory not found"))?;
        let current_versions_path = desktop.join("swuc_current.txt");

        // Generate changelog
        self.generate_changelog(&current_versions_path)?;

        Ok(())
    }

    fn generate_changelog(&mut self, current_versions_path: &Path) -> Result<()> {
        // Load saved versions
        let saved_versions = self.read_saved_versions(current_versions_path);
        let mut new_versions = Vec::new(); // To preserve order from JSON

        for package in &self.packages {
            let name = &package.name;
            let new_version = &package.version;
            new_versions.push((name.clone(), new_version.clone()));

            let old_version = saved_versions.get(name);
            if old_version != Some(new_version) {
                // Format short changelog entry
                let old_display = old_version.map(String::as_str).unwrap_or("");
                self.short_changelog
                    .push_str(&format!("{} {} -> {}\n", name, old_display, new_version));

                // Format long changelog entry
                self.long_changelog
                    .push_str(&format!("{} {} -> {}\n", name, old_display, new_version));
                self.long_changelog.push_str("| - Sources:\n");
                for source in &package.sources {
                    self.long_changelog
                        .push_str(&format!("    | - {}\n", source));
                }
                self.long_changelog.push('\n');
            }
        }

        // Save new versions to swuc_current.txt
        let new_versions_content: String = new_versions
            .iter()
            .map(|(name, version)| format!("{}: {}\n", name, version))
            .collect();

        fs::write(current_versions_path, new_versions_content).with_context(|| {
            format!(
                "Failed to write current versions to {:?}",
                current_versions_path
            )
        })?;

        // Save long changelog if there are changes
        if !self.long_changelog.is_empty() {
            let desktop =
                desktop_dir().ok_or_else(|| anyhow::anyhow!("Desktop directory not found"))?;
            let reports_dir = desktop.join("swuc-reports");
            fs::create_dir_all(&reports_dir).with_context(|| {
                format!("Failed to create reports directory at {:?}", reports_dir)
            })?;

            // Write to latest.txt
            let latest_path = reports_dir.join("latest.txt");
            fs::write(&latest_path, &self.long_changelog)
                .with_context(|| format!("Failed to write latest report to {:?}", latest_path))?;

            // Write timestamped report
            let now = Local::now();
            let timestamp = now.format("%H-%M_%d-%m-%Y").to_string();
            let report_path = reports_dir.join(format!("report_{}.txt", timestamp));
            fs::write(&report_path, &self.long_changelog).with_context(|| {
                format!("Failed to write timestamped report to {:?}", report_path)
            })?;
        }

        Ok(())
    }

    fn read_saved_versions(&self, path: &Path) -> HashMap<String, String> {
        let mut versions = HashMap::new();
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.splitn(2, ": ").collect();
                if parts.len() == 2 {
                    versions.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
        versions
    }

    pub fn short_report(&self) -> &str {
        &self.short_changelog
    }

    pub fn full_report(&self) -> &str {
        &self.long_changelog
    }

    // New method to save report directly to a specified path
    pub fn save_report_to(&self, path: &str) -> Result<()> {
        let report = format!(
            "Update report generated at {}\n\n{}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            self.full_report()
        );

        fs::write(path, report).with_context(|| format!("Failed to write report to {}", path))?;

        Ok(())
    }
}
