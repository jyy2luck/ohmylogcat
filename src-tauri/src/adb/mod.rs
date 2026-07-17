pub mod stream;

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub serial: String,
    pub state: String,
}

/// Common ADB locations per-platform for fallback.
fn platform_default_paths() -> Vec<String> {
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        // Homebrew
        paths.push("/opt/homebrew/bin/adb".into());
        paths.push("/usr/local/bin/adb".into());
        // Android SDK common locations
        let home = std::env::var("HOME").unwrap_or_default();
        paths.push(format!("{}/Library/Android/sdk/platform-tools/adb", home));
        paths.push(format!("{}/Android/Sdk/platform-tools/adb", home));
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            paths.push(format!(
                "{}\\Android\\Sdk\\platform-tools\\adb.exe",
                localappdata
            ));
        }
        if let Ok(program_files) = std::env::var("ProgramFiles(x86)") {
            paths.push(format!("{}\\Android\\android-sdk\\platform-tools\\adb.exe", program_files));
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        paths.push("/usr/bin/adb".into());
        paths.push("/usr/local/bin/adb".into());
        paths.push(format!("{}/Android/Sdk/platform-tools/adb", home));
    }

    paths
}

/// Resolve adb executable path.
/// First checks the configured path (if provided), then PATH, then well-known locations.
pub fn resolve_adb_path(configured: Option<&str>) -> Result<String, String> {
    // Try configured path first
    if let Some(path) = configured {
        if !path.is_empty() {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
            return Err(format!("ADB not found at configured path: {}", path));
        }
    }

    // Check PATH
    if let Ok(path) = which_adb() {
        return Ok(path);
    }

    // Fallback: check well-known locations
    for p in platform_default_paths() {
        if std::path::Path::new(&p).exists() {
            return Ok(p);
        }
    }

    Err("ADB not found. Install Android SDK platform-tools or configure adb path in settings.".into())
}

/// Check if adb is in PATH and return its path.
fn which_adb() -> Result<String, String> {
    let output = Command::new("which").arg("adb").output().map_err(|e| e.to_string())?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path.is_empty() && std::path::Path::new(&path).exists() {
            return Ok(path);
        }
    }
    Err("adb not found in PATH".into())
}

/// Run `adb --version` to verify the binary works and return version info.
pub fn check_adb_version(adb_path: &str) -> Result<String, String> {
    let output = Command::new(adb_path)
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute adb: {}", e))?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if version.is_empty() {
            // adb sometimes outputs version on stderr
            Ok(String::from_utf8_lossy(&output.stderr).trim().to_string())
        } else {
            Ok(version)
        }
    } else {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!("ADB version check failed: {}", err))
    }
}

/// Run `adb devices` and return parsed device list.
pub fn list_devices(adb_path: &str) -> Result<Vec<Device>, String> {
    let output = Command::new(adb_path)
        .arg("devices")
        .output()
        .map_err(|e| format!("Failed to execute adb devices: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("adb devices failed: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines().skip(1) {
        // Skip the "List of devices attached" header
        if line.contains("List of") || line.trim().is_empty() {
            continue;
        }
        // Format: "emulator-5554\tdevice" or "0123456789abcdef\tdevice"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            devices.push(Device {
                serial: parts[0].to_string(),
                state: parts[1].to_string(),
            });
        }
    }

    Ok(devices)
}
