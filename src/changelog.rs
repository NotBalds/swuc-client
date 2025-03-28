use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Software {
    pub name: String,
    pub version: String,
    pub raw_version_info: String,
    pub token_usage: u32,
    pub token_cost: f64,
    pub sources: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SoftwareResponse {
    pub status: String,
    pub software: Vec<Software>,
}

#[derive(Debug)]
pub struct ChangelogEntry {
    pub name: String,
    pub old_version: String,
    pub new_version: String,
    pub sources: Vec<String>,
}

pub struct ChangelogGenerator {
    version_db: HashMap<String, String>, // Simplified to just name -> version
    software_db: HashMap<String, Software>, // Full software info
    changelog_entries: Vec<ChangelogEntry>,
}

impl ChangelogGenerator {
    pub fn new() -> Self {
        ChangelogGenerator {
            version_db: HashMap::new(),
            software_db: HashMap::new(),
            changelog_entries: Vec::new(),
        }
    }

    pub fn load_existing_versions(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        if let Ok(contents) = fs::read_to_string(path) {
            match serde_json::from_str::<HashMap<String, String>>(&contents) {
                Ok(versions) => {
                    self.version_db = versions;
                    Ok(())
                }
                Err(e) => Err(format!("Failed to parse software.json: {}", e).into()),
            }
        } else {
            println!("No existing version file found. Starting fresh.");
            Ok(())
        }
    }

    pub fn save_updated_versions(&self, path: &str) -> Result<(), Box<dyn Error>> {
        // Create a new HashMap with just the current versions
        let mut current_versions = HashMap::new();
        for (name, software) in &self.software_db {
            current_versions.insert(name.clone(), software.version.clone());
        }

        let json = serde_json::to_string_pretty(&current_versions)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn parse_response(&mut self, json_str: &str) -> Result<(), Box<dyn Error>> {
        let response: SoftwareResponse = serde_json::from_str(json_str)?;

        if response.status != "success" {
            return Err("Response status is not 'success'".into());
        }

        // Process each software entry
        for software in response.software {
            if let Some(existing_version) = self.version_db.get(&software.name) {
                // If the version is different, create a changelog entry
                if existing_version != &software.version && software.version != "Unknown" {
                    self.changelog_entries.push(ChangelogEntry {
                        name: software.name.clone(),
                        old_version: existing_version.clone(),
                        new_version: software.version.clone(),
                        sources: software.sources.clone(),
                    });
                }
            }

            // Update or add the software to the database
            if software.version != "Unknown" {
                self.version_db
                    .insert(software.name.clone(), software.version.clone());
                self.software_db.insert(software.name.clone(), software);
            }
        }

        Ok(())
    }

    pub fn generate_short_changelog(&self) -> String {
        if self.changelog_entries.is_empty() {
            return "No changes detected.".to_string();
        }

        let mut changelog = String::new();

        for entry in &self.changelog_entries {
            changelog.push_str(&format!(
                "{}: {} -> {}\n",
                entry.name, entry.old_version, entry.new_version
            ));
        }

        changelog
    }

    pub fn generate_full_changelog(&self) -> String {
        if self.changelog_entries.is_empty() {
            return "No changes detected.".to_string();
        }

        let mut changelog = String::new();

        for entry in &self.changelog_entries {
            changelog.push_str(&format!("[{}]\n", entry.name));
            changelog.push_str(&format!("old_version = \"{}\"\n", entry.old_version));
            changelog.push_str(&format!("new_version = \"{}\"\n", entry.new_version));
            changelog.push_str("sources = [\n");

            for source in &entry.sources {
                changelog.push_str(&format!("  \"{}\",\n", source));
            }

            changelog.push_str("]\n\n");
        }

        changelog
    }

    pub fn save_full_changelog(&self) -> Result<(), Box<dyn Error>> {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let filename = format!("{}-changelog.toml", date);

        let changelog = self.generate_full_changelog();
        fs::write(&filename, changelog)?;

        println!("Full changelog saved to {}", filename);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let json_str = r#"{"status": "success", "software": [{"name": "cwe-client-cli", "version": "0.3.2", "raw_version_info": "\u0412\u0435\u0440\u0441\u0438\u044f cwe-client-cli \u2014 0.3.2.", "token_usage": 887, "token_cost": 0.177, "sources": ["https://aur.archlinux.org/packages/cwe-client-cli", "https://github.com/NotBalds/cwe-client-cli", "https://mynixos.com/nixpkgs/package/cwe-client-cli"]}, {"name": "android", "version": "Unknown", "raw_version_info": "Android 15", "token_usage": 785, "token_cost": 0.157, "sources": ["https://trainings.internshala.com/blog/android-versions/", "https://en.wikipedia.org/wiki/Android_version_history", "https://developer.android.com/about/versions"]}, {"name": "rustlang", "version": "1.45.2", "raw_version_info": "\u0412\u0435\u0440\u0441\u0438\u044f Rustlang \u2014 1.45.2.", "token_usage": 758, "token_cost": 0.152, "sources": ["https://www.rust-lang.org/", "https://github.com/rust-lang/rust/releases", "https://wingetgui.com/apps/Rustlang-rust-gnu-x64"]}, {"name": "nginx", "version": "1.27.4", "raw_version_info": "The most recent version information for nginx is 1.27.4.", "token_usage": 914, "token_cost": 0.183, "sources": ["https://docs.nginx.com/nginx/releases/", "https://nginx.org/en/download.html", "https://github.com/nginx/nginx/releases"]}]}"#;

    // Create version database for testing
    if !fs::metadata("software.json").is_ok() {
        let test_versions = HashMap::from([
            ("cwe-client-cli".to_string(), "0.3.1".to_string()),
            ("rustlang".to_string(), "1.45.0".to_string()),
            ("nginx".to_string(), "1.27.3".to_string()),
        ]);
        let json = serde_json::to_string_pretty(&test_versions)?;
        fs::write("software.json", json)?;
        println!("Created test software.json file");
    }

    // Initialize generator
    let mut generator = ChangelogGenerator::new();

    // Load existing versions
    generator.load_existing_versions("software.json")?;

    // Parse new response
    generator.parse_response(json_str)?;

    // Generate and print short changelog
    let short_changelog = generator.generate_short_changelog();
    println!("Short Changelog:\n{}", short_changelog);

    // Save full changelog to a file
    generator.save_full_changelog()?;

    // Save updated versions
    generator.save_updated_versions("software.json")?;

    Ok(())
}
