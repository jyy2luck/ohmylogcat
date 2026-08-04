//! Out-of-TUI lifecycle commands: version, help, update, uninstall.

use crate::settings;
use std::env;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO: &str = "jyy2luck/ohmylogcat";
#[cfg(not(windows))]
const INSTALL_SH_URL: &str =
    "https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.sh";
#[cfg(windows)]
const INSTALL_PS1_URL: &str =
    "https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.ps1";
const RELEASES_API: &str =
    "https://api.github.com/repos/jyy2luck/ohmylogcat/releases/latest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    RunTui,
    Version,
    Help,
    Update,
    Uninstall {
        yes: bool,
        keep_data: bool,
        purge: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parse CLI args (without program name).
pub fn parse<I, S>(args: I) -> Result<CliAction, ParseError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter();
    let Some(first) = iter.next() else {
        return Ok(CliAction::RunTui);
    };
    let first = first.as_ref();

    match first {
        "--version" | "-V" => {
            if iter.next().is_some() {
                return Err(ParseError::new(
                    "unexpected arguments after --version\nTry `ohmylogcat --help` for usage.",
                ));
            }
            Ok(CliAction::Version)
        }
        "--help" | "-h" => {
            if iter.next().is_some() {
                return Err(ParseError::new(
                    "unexpected arguments after --help\nTry `ohmylogcat --help` for usage.",
                ));
            }
            Ok(CliAction::Help)
        }
        "update" => {
            if iter.next().is_some() {
                return Err(ParseError::new(
                    "unexpected arguments after `update`\nTry `ohmylogcat --help` for usage.",
                ));
            }
            Ok(CliAction::Update)
        }
        "uninstall" => parse_uninstall(iter),
        other => Err(ParseError::new(format!(
            "unknown argument: {other}\nTry `ohmylogcat --help` for usage."
        ))),
    }
}

fn parse_uninstall<I, S>(args: I) -> Result<CliAction, ParseError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut yes = false;
    let mut keep_data = false;
    let mut purge = false;

    for arg in args {
        match arg.as_ref() {
            "--yes" | "-y" => yes = true,
            "--keep-data" => keep_data = true,
            "--purge" => purge = true,
            other => {
                return Err(ParseError::new(format!(
                    "unknown uninstall flag: {other}\nTry `ohmylogcat --help` for usage."
                )));
            }
        }
    }

    if keep_data && purge {
        return Err(ParseError::new(
            "cannot combine --keep-data and --purge\nTry `ohmylogcat --help` for usage.",
        ));
    }

    Ok(CliAction::Uninstall {
        yes,
        keep_data,
        purge,
    })
}

pub fn version_text() -> String {
    format!("ohmylogcat {PKG_VERSION}")
}

pub fn help_text() -> String {
    format!(
        "\
ohmylogcat {PKG_VERSION} — lightweight Android Logcat TUI

USAGE:
    ohmylogcat              Start the TUI
    ohmylogcat --version    Print version (-V)
    ohmylogcat --help       Show this help (-h)
    ohmylogcat update       Re-run the install script (latest GitHub Release)
    ohmylogcat uninstall    Remove a Release-script install

UNINSTALL OPTIONS:
    -y, --yes       Skip confirmation
    --keep-data     Keep settings.json (no prompt)
    --purge         Delete settings.json (no keep prompt)

COMMON SHORTCUTS (in TUI):
    q         Quit (top layer)
    d         Device list
    Space     Pause / Resume
    c         Clear buffer
    f         Toggle Follow (tail)
    t / m     Tag / Message filter
    /         Find
    s         Settings
    e         Export

See README for the full shortcut table:
  https://github.com/{REPO}#keyboard-shortcuts
"
    )
}

/// Run a non-TUI action. Returns an exit code on failure.
pub fn run(action: CliAction) -> Result<(), i32> {
    match action {
        CliAction::RunTui => Ok(()),
        CliAction::Version => {
            println!("{}", version_text());
            Ok(())
        }
        CliAction::Help => {
            print!("{}", help_text());
            Ok(())
        }
        CliAction::Update => run_update(),
        CliAction::Uninstall {
            yes,
            keep_data,
            purge,
        } => run_uninstall(yes, keep_data, purge),
    }
}

fn run_update() -> Result<(), i32> {
    let exe = current_exe().map_err(|e| {
        eprintln!("error: {e}");
        1
    })?;

    if !is_release_install(&exe) {
        eprintln!(
            "This binary does not look like a Release-script install ({})",
            exe.display()
        );
        eprintln!("To update a cargo install, run:");
        eprintln!("  cargo install --git https://github.com/{REPO} --force");
        return Err(1);
    }

    println!("Updating ohmylogcat via install script...");
    invoke_install_script().map_err(|e| {
        eprintln!("error: {e}");
        1
    })?;

    match fetch_latest_release_version() {
        Ok(latest) if latest == PKG_VERSION => {
            println!("Already up to date ({latest}).");
        }
        Ok(latest) => {
            println!("Updated toward latest release ({latest}). Restart any open sessions.");
        }
        Err(e) => {
            eprintln!("warning: could not verify latest release version: {e}");
            println!("Install script finished. Run `ohmylogcat --version` to confirm.");
        }
    }

    Ok(())
}

fn run_uninstall(yes: bool, keep_data: bool, purge: bool) -> Result<(), i32> {
    let exe = current_exe().map_err(|e| {
        eprintln!("error: {e}");
        1
    })?;

    if !is_release_install(&exe) {
        eprintln!(
            "This binary does not look like a Release-script install ({})",
            exe.display()
        );
        eprintln!("To uninstall a cargo install, run:");
        eprintln!("  cargo uninstall ohmylogcat");
        return Err(1);
    }

    let install_dir = release_install_dir();
    let binary = release_binary_path();
    let tty = io::stdin().is_terminal();

    if !yes {
        if !tty {
            eprintln!("error: non-interactive uninstall requires --yes (-y)");
            return Err(1);
        }
        if !confirm(&format!(
            "Remove {}? [y/N] ",
            binary.display()
        ))
        .map_err(|e| {
            eprintln!("error: {e}");
            1
        })? {
            println!("Aborted.");
            return Ok(());
        }
    }

    let should_purge = if purge {
        true
    } else if keep_data {
        false
    } else if tty {
        !confirm("Keep settings (settings.json)? [Y/n] ").map_err(|e| {
            eprintln!("error: {e}");
            1
        })?
    } else {
        // Non-TTY without keep/purge: safe default is keep.
        false
    };

    remove_release_binary(&binary).map_err(|e| {
        eprintln!("error: failed to remove binary: {e}");
        1
    })?;

    #[cfg(windows)]
    {
        if let Err(e) = remove_windows_user_path_entry(&install_dir) {
            eprintln!("warning: could not update user PATH: {e}");
        }
    }
    #[cfg(not(windows))]
    {
        let _ = install_dir;
    }

    if should_purge {
        if let Err(e) = purge_settings() {
            eprintln!("warning: could not purge settings: {e}");
        } else {
            println!("Removed settings.");
        }
    } else {
        println!("Settings kept.");
    }

    println!("Uninstalled ohmylogcat.");
    Ok(())
}

fn confirm(prompt: &str) -> io::Result<bool> {
    eprint!("{prompt}");
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        // Empty answer: yes for [Y/n], no for [y/N]
        return Ok(prompt.contains("[Y/n]"));
    }
    let lower = trimmed.to_ascii_lowercase();
    Ok(lower == "y" || lower == "yes")
}

fn current_exe() -> Result<PathBuf, String> {
    env::current_exe().map_err(|e| e.to_string())
}

fn release_install_dir() -> PathBuf {
    if let Ok(dir) = env::var("INSTALL_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    #[cfg(windows)]
    {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from(r"C:\Users\Public"))
            .join("ohmylogcat")
    }
    #[cfg(not(windows))]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/"))
            .join(".local")
            .join("bin")
    }
}

fn release_binary_path() -> PathBuf {
    let dir = release_install_dir();
    #[cfg(windows)]
    {
        dir.join("ohmylogcat.exe")
    }
    #[cfg(not(windows))]
    {
        dir.join("ohmylogcat")
    }
}

fn is_release_install(exe: &Path) -> bool {
    let expected = release_binary_path();
    same_path(exe, &expected)
}

fn same_path(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(ca), Ok(cb)) => ca == cb,
        _ => {
            // Fallback: compare as strings after normalizing separators.
            let na = normalize_path_string(a);
            let nb = normalize_path_string(b);
            na.eq_ignore_ascii_case(&nb)
        }
    }
}

fn normalize_path_string(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches(['\\', '/'])
        .to_string()
}

fn invoke_install_script() -> Result<(), String> {
    #[cfg(windows)]
    {
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &format!("irm {INSTALL_PS1_URL} | iex"),
            ])
            .status()
            .map_err(|e| format!("failed to run PowerShell: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("install script exited with {status}"))
        }
    }
    #[cfg(not(windows))]
    {
        let status = Command::new("sh")
            .args(["-c", &format!("curl -fsSL {INSTALL_SH_URL} | sh")])
            .status()
            .map_err(|e| format!("failed to run install script: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("install script exited with {status}"))
        }
    }
}

fn fetch_latest_release_version() -> Result<String, String> {
    let body = fetch_url(RELEASES_API)?;
    let tag = extract_json_string_field(&body, "tag_name")
        .ok_or_else(|| "missing tag_name in release metadata".to_string())?;
    Ok(tag.trim_start_matches('v').to_string())
}

fn fetch_url(url: &str) -> Result<String, String> {
    #[cfg(windows)]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "$ProgressPreference='SilentlyContinue'; \
                     (Invoke-WebRequest -Uri '{url}' -Headers @{{'User-Agent'='ohmylogcat';'Accept'='application/vnd.github+json'}} -UseBasicParsing).Content"
                ),
            ])
            .output()
            .map_err(|e| format!("failed to fetch {url}: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("failed to fetch {url}: {stderr}"));
        }
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        let output = Command::new("curl")
            .args([
                "-fsSL",
                "-H",
                "User-Agent: ohmylogcat",
                "-H",
                "Accept: application/vnd.github+json",
                url,
            ])
            .output()
            .map_err(|e| format!("failed to fetch {url}: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("failed to fetch {url}: {stderr}"));
        }
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    }
}

fn extract_json_string_field(json: &str, field: &str) -> Option<String> {
    let key = format!("\"{field}\"");
    let idx = json.find(&key)?;
    let after = &json[idx + key.len()..];
    let after = after.trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    let after = after.strip_prefix('"')?;
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

fn remove_release_binary(binary: &Path) -> Result<(), String> {
    if !binary.exists() {
        return Err(format!("binary not found: {}", binary.display()));
    }

    #[cfg(windows)]
    {
        match fs::remove_file(binary) {
            Ok(()) => {
                try_remove_empty_dir(binary.parent());
                Ok(())
            }
            Err(_) => {
                // Running exe is often locked on Windows — schedule delayed delete.
                let path = binary.display().to_string();
                let cmd = format!(
                    "ping 127.0.0.1 -n 2 > nul & del /F /Q \"{path}\" & \
                     if exist \"{path}\" exit 1"
                );
                Command::new("cmd")
                    .args(["/C", &cmd])
                    .spawn()
                    .map_err(|e| format!("failed to schedule binary deletion: {e}"))?;
                eprintln!("Scheduled removal of locked binary; it will disappear shortly.");
                Ok(())
            }
        }
    }
    #[cfg(not(windows))]
    {
        fs::remove_file(binary).map_err(|e| e.to_string())?;
        try_remove_empty_dir(binary.parent());
        Ok(())
    }
}

fn try_remove_empty_dir(dir: Option<&Path>) {
    if let Some(dir) = dir {
        let _ = fs::remove_dir(dir);
    }
}

#[cfg(windows)]
fn remove_windows_user_path_entry(install_dir: &Path) -> Result<(), String> {
    let dir = install_dir
        .canonicalize()
        .unwrap_or_else(|_| install_dir.to_path_buf());
    let dir_str = dir.to_string_lossy().trim_end_matches('\\').to_string();

    let script = format!(
        r#"
$dir = '{dir_str}'
$path = [Environment]::GetEnvironmentVariable('Path', 'User')
if (-not $path) {{ exit 0 }}
$entries = $path -split ';' | Where-Object {{ $_ -and ($_.TrimEnd('\') -ne $dir) }}
$newPath = ($entries -join ';')
[Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
"#
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("PATH cleanup exited with {status}"))
    }
}

fn purge_settings() -> Result<(), String> {
    let path = settings::config_path()?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir(parent);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_no_args_runs_tui() {
        let action = parse(Vec::<&str>::new()).unwrap();
        assert_eq!(action, CliAction::RunTui);
    }

    #[test]
    fn parse_version_flags() {
        assert_eq!(parse(["--version"]).unwrap(), CliAction::Version);
        assert_eq!(parse(["-V"]).unwrap(), CliAction::Version);
    }

    #[test]
    fn parse_help_flags() {
        assert_eq!(parse(["--help"]).unwrap(), CliAction::Help);
        assert_eq!(parse(["-h"]).unwrap(), CliAction::Help);
    }

    #[test]
    fn parse_update() {
        assert_eq!(parse(["update"]).unwrap(), CliAction::Update);
    }

    #[test]
    fn parse_uninstall_flags() {
        assert_eq!(
            parse(["uninstall"]).unwrap(),
            CliAction::Uninstall {
                yes: false,
                keep_data: false,
                purge: false,
            }
        );
        assert_eq!(
            parse(["uninstall", "--yes", "--keep-data"]).unwrap(),
            CliAction::Uninstall {
                yes: true,
                keep_data: true,
                purge: false,
            }
        );
        assert_eq!(
            parse(["uninstall", "-y", "--purge"]).unwrap(),
            CliAction::Uninstall {
                yes: true,
                keep_data: false,
                purge: true,
            }
        );
    }

    #[test]
    fn parse_rejects_unknown_and_conflicts() {
        assert!(parse(["nope"]).is_err());
        assert!(parse(["uninstall", "--keep-data", "--purge"]).is_err());
        assert!(parse(["uninstall", "--bogus"]).is_err());
        let err = parse(["wat"]).unwrap_err();
        assert!(err.message.contains("--help"));
    }

    #[test]
    fn version_and_help_content() {
        let v = version_text();
        assert!(v.contains(PKG_VERSION));
        assert!(v.starts_with("ohmylogcat "));

        let h = help_text();
        assert!(h.contains("update"));
        assert!(h.contains("uninstall"));
        assert!(h.contains("--version"));
        assert!(h.contains("--help"));
        assert!(h.contains("Start the TUI"));
        assert!(h.contains('q'));
        assert!(h.contains("README"));
    }

    #[test]
    fn extract_tag_name() {
        let json = r#"{"tag_name": "v0.1.0", "name": "x"}"#;
        assert_eq!(
            extract_json_string_field(json, "tag_name").as_deref(),
            Some("v0.1.0")
        );
    }
}
