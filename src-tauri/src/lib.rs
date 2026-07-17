#![allow(dead_code)]

mod adb;
mod buffer;
mod engine;
mod filter;
mod parser;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Manager, State};

pub use engine::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub adb_path: Option<String>,
    pub buffer_capacity: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            adb_path: None,
            buffer_capacity: 200_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BufferStats {
    pub lines_per_sec: f64,
    pub memory_estimate_mb: f64,
    pub count: usize,
    pub capacity: usize,
}

// ── ADB Commands ──

#[tauri::command]
fn list_devices(_engine: State<Arc<Engine>>) -> Result<Vec<adb::Device>, String> {
    let adb_path = adb::resolve_adb_path(None)?;
    adb::list_devices(&adb_path)
}

#[tauri::command]
fn check_adb() -> Result<String, String> {
    let adb_path = adb::resolve_adb_path(None)?;
    adb::check_adb_version(&adb_path)
}

// ── Stream Commands ──

#[tauri::command]
fn start_stream(
    app_handle: tauri::AppHandle,
    engine: State<Arc<Engine>>,
    serial: String,
) -> Result<(), String> {
    let adb_path = adb::resolve_adb_path(None)?;
    engine.start_stream(app_handle, adb_path, serial);
    Ok(())
}

#[tauri::command]
fn stop_stream(engine: State<Arc<Engine>>) {
    engine.stop_stream();
}

#[tauri::command]
fn pause_stream(engine: State<Arc<Engine>>) {
    *engine.is_paused.lock().unwrap() = true;
}

#[tauri::command]
fn resume_stream(engine: State<Arc<Engine>>) {
    *engine.is_paused.lock().unwrap() = false;
}

#[tauri::command]
fn clear_buffer(app_handle: tauri::AppHandle, engine: State<Arc<Engine>>) {
    engine.clear_buffer(&app_handle);
}

#[tauri::command]
fn get_filtered_logs(app_handle: tauri::AppHandle, engine: State<Arc<Engine>>) {
    engine.re_filter(&app_handle);
}

// ── Filter Commands ──

#[tauri::command]
fn set_filter(
    app_handle: tauri::AppHandle,
    engine: State<Arc<Engine>>,
    tag: Option<String>,
    message: Option<String>,
    level: Option<String>,
) {
    let min_level = level.as_deref().and_then(|l| {
        use crate::parser::LogLevel;
        match l {
            "Verbose" => Some(LogLevel::Verbose),
            "Debug" => Some(LogLevel::Debug),
            "Info" => Some(LogLevel::Info),
            "Warn" => Some(LogLevel::Warn),
            "Error" => Some(LogLevel::Error),
            "Fatal" => Some(LogLevel::Fatal),
            _ => None,
        }
    });

    let mut filter = engine.filter.lock().unwrap();
    filter.tag_substring = tag;
    filter.message_substring = message;
    filter.min_level = min_level;
    drop(filter);
    engine.re_filter(&app_handle);
}

// ── Export Commands ──

#[tauri::command]
fn export_logs(
    engine: State<Arc<Engine>>,
    file_path: String,
    filtered_only: bool,
) -> Result<(), String> {
    use std::io::Write;

    let entries: Vec<crate::parser::LogEntry> = {
        let buffer = engine.buffer.lock().unwrap();
        if filtered_only {
            let filter = engine.filter.lock().unwrap();
            buffer.iter().filter(|e| filter.matches(e)).cloned().collect()
        } else {
            buffer.iter().cloned().collect()
        }
    };

    if entries.is_empty() {
        return Err("No log entries to export".into());
    }

    let mut file =
        std::fs::File::create(&file_path).map_err(|e| format!("Failed to create file: {}", e))?;

    for entry in &entries {
        let line = format!(
            "{} {:5} {:5} {} {}: {}\n",
            entry.timestamp,
            entry.pid,
            entry.tid,
            entry.level.to_display(),
            entry.tag,
            entry.message
        );
        file.write_all(line.as_bytes())
            .map_err(|e| format!("Failed to write: {}", e))?;
    }

    Ok(())
}

// ── Settings Commands ──

#[tauri::command]
fn load_settings(
    app_handle: tauri::AppHandle,
    engine: State<Arc<Engine>>,
) -> Result<Settings, String> {
    let config_path = app_handle
        .path()
        .resolve("settings.json", tauri::path::BaseDirectory::AppConfig)
        .map_err(|e| e.to_string())?;

    if config_path.exists() {
        let data = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let settings: Settings = serde_json::from_str(&data).map_err(|e| e.to_string())?;
        let cap = settings.buffer_capacity;
        let mut bufcap = engine.settings_bufcap.lock().unwrap();
        *bufcap = cap;
        engine.buffer.lock().unwrap().set_capacity(cap);
        Ok(settings)
    } else {
        Ok(Settings::default())
    }
}

#[tauri::command]
fn save_settings(
    app_handle: tauri::AppHandle,
    engine: State<Arc<Engine>>,
    settings: Settings,
) -> Result<(), String> {
    let config_path = app_handle
        .path()
        .resolve("settings.json", tauri::path::BaseDirectory::AppConfig)
        .map_err(|e| e.to_string())?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let data =
        serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, data).map_err(|e| e.to_string())?;

    let mut bufcap = engine.settings_bufcap.lock().unwrap();
    *bufcap = settings.buffer_capacity;
    engine.buffer.lock().unwrap().set_capacity(settings.buffer_capacity);

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let engine = Engine::new(200_000);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            list_devices,
            check_adb,
            start_stream,
            stop_stream,
            pause_stream,
            resume_stream,
            clear_buffer,
            get_filtered_logs,
            set_filter,
            export_logs,
            load_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
