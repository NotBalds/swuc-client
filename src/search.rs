use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

pub type AppVersions = HashMap<String, String>;

pub struct AppFinder {
    platform: String,
}

impl AppFinder {
    /// Create a new AppFinder for the current platform
    pub fn new() -> Self {
        let platform = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        }
        .to_string();

        AppFinder { platform }
    }

    /// Get installed applications and their versions
    pub fn get_installed_apps(&self) -> Result<AppVersions, Box<dyn Error>> {
        match self.platform.as_str() {
            "windows" => self.get_windows_apps(),
            "macos" => self.get_macos_apps(),
            "linux" => self.get_linux_apps(),
            _ => Err("Unsupported platform".into()),
        }
    }

    /// Get installed apps on Windows
    fn get_windows_apps(&self) -> Result<AppVersions, Box<dyn Error>> {
        let mut apps = AppVersions::new();

        // Using PowerShell to get installed apps from registry
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("Get-ItemProperty HKLM:\\Software\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Select-Object DisplayName, DisplayVersion | ConvertTo-Csv -NoTypeInformation")
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let lines: Vec<&str> = stdout.lines().collect();

        // Skip header line
        for line in lines.iter().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                // Remove quotes from csv output
                let name = parts[0].trim_matches('"');
                let version = parts[1].trim_matches('"');

                if !name.is_empty() {
                    apps.insert(name.to_string(), version.to_string());
                }
            }
        }

        // Also check the 64-bit registry key
        let output = Command::new("powershell")
            .arg("-Command")
            .arg("Get-ItemProperty HKLM:\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\* | Select-Object DisplayName, DisplayVersion | ConvertTo-Csv -NoTypeInformation")
            .output()?;

        let stdout = String::from_utf8(output.stdout)?;
        let lines: Vec<&str> = stdout.lines().collect();

        for line in lines.iter().skip(1) {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 2 {
                let name = parts[0].trim_matches('"');
                let version = parts[1].trim_matches('"');

                if !name.is_empty() {
                    apps.insert(name.to_string(), version.to_string());
                }
            }
        }

        Ok(apps)
    }

    /// Get installed apps on macOS
    fn get_macos_apps(&self) -> Result<AppVersions, Box<dyn Error>> {
        let mut apps = AppVersions::new();

        // Check /Applications folder
        if let Ok(entries) = fs::read_dir("/Applications") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| ext == "app") {
                        let app_name = path.file_stem().unwrap().to_string_lossy().to_string();

                        // Try to get version from Info.plist
                        let plist_path = path.join("Contents/Info.plist");
                        if plist_path.exists() {
                            let output = Command::new("defaults")
                                .arg("read")
                                .arg(plist_path.to_string_lossy().to_string())
                                .arg("CFBundleShortVersionString")
                                .output();

                            if let Ok(output) = output {
                                if output.status.success() {
                                    let version =
                                        String::from_utf8_lossy(&output.stdout).trim().to_string();
                                    apps.insert(app_name, version);
                                    continue;
                                }
                            }

                            // Try CFBundleVersion if CFBundleShortVersionString doesn't exist
                            let output = Command::new("defaults")
                                .arg("read")
                                .arg(plist_path.to_string_lossy().to_string())
                                .arg("CFBundleVersion")
                                .output();

                            if let Ok(output) = output {
                                if output.status.success() {
                                    let version =
                                        String::from_utf8_lossy(&output.stdout).trim().to_string();
                                    apps.insert(app_name, version);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also check homebrew installed apps
        if Path::new("/usr/local/bin/brew").exists() || Path::new("/opt/homebrew/bin/brew").exists()
        {
            let output = Command::new("brew").arg("list").arg("--versions").output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let name = parts[0].to_string();
                            let version = parts[1].to_string();
                            apps.insert(name, version);
                        }
                    }
                }
            }
        }

        Ok(apps)
    }

    /// Get installed apps on Linux
    fn get_linux_apps(&self) -> Result<AppVersions, Box<dyn Error>> {
        let mut apps = AppVersions::new();

        // Try different package managers

        // Debian/Ubuntu (apt)
        if Path::new("/usr/bin/dpkg").exists() {
            let output = Command::new("dpkg").arg("-l").output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        if line.starts_with("ii") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 3 {
                                let name = parts[1].to_string();
                                let version = parts[2].to_string();
                                apps.insert(name, version);
                            }
                        }
                    }
                }
            }
        }

        // Red Hat/Fedora/CentOS (rpm)
        if Path::new("/usr/bin/rpm").exists() {
            let output = Command::new("rpm")
                .arg("-qa")
                .arg("--queryformat")
                .arg("%{NAME},%{VERSION}-%{RELEASE}\n")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split(',').collect();
                        if parts.len() == 2 {
                            let name = parts[0].to_string();
                            let version = parts[1].to_string();
                            apps.insert(name, version);
                        }
                    }
                }
            }
        }

        // Arch Linux (pacman)
        if Path::new("/usr/bin/pacman").exists() {
            let output = Command::new("pacman").arg("-Q").output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() == 2 {
                            let name = parts[0].to_string();
                            let version = parts[1].to_string();
                            apps.insert(name, version);
                        }
                    }
                }
            }
        }

        // Flatpak
        if Path::new("/usr/bin/flatpak").exists() {
            let output = Command::new("flatpak")
                .arg("list")
                .arg("--app")
                .arg("--columns=application,version")
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let name = parts[0].to_string();
                            let version = parts[1].to_string();
                            apps.insert(name, version);
                        }
                    }
                }
            }
        }

        // Snap
        if Path::new("/usr/bin/snap").exists() {
            let output = Command::new("snap").arg("list").output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // Skip header line
                    for line in stdout.lines().skip(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let name = parts[0].to_string();
                            let version = parts[1].to_string();
                            apps.insert(name, version);
                        }
                    }
                }
            }
        }

        Ok(apps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_finder_new() {
        let finder = AppFinder::new();
        assert!(["windows", "macos", "linux", "unknown"].contains(&finder.platform.as_str()));
    }
}
