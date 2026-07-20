// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adb;
mod app;
mod buffer;
mod engine;
mod filter;
mod parser;
mod settings;
mod ui;

use app::OhmylogcatApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([640.0, 400.0])
            .with_title("Oh My Logcat")
            .with_active(true),
        ..Default::default()
    };

    eprintln!("Oh My Logcat starting — look for the app window (Dock / Mission Control).");

    eframe::run_native(
        "Oh My Logcat",
        options,
        Box::new(|cc| Ok(Box::new(OhmylogcatApp::new(cc)))),
    )
}
